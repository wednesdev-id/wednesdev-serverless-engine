#!/usr/bin/env python3
"""Linux-local benchmark: owns daemon, measures /proc, validates HTTP results."""
import argparse
import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import socket
import subprocess
import threading
import time
import urllib.request
import urllib.error


def percentiles(values):
    values = sorted(values)
    return {f'p{p}_ms': values[max(0, math.ceil(len(values)*p/100)-1)] for p in (50,95,99)}


def proc_values(path):
    return {line.split()[0].rstrip(':'): int(line.split()[1]) for line in Path(path).read_text().splitlines() if len(line.split()) > 1 and line.split()[1].isdigit()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--binary', required=True)
    ap.add_argument('--artifacts', required=True)
    ap.add_argument('--output', default='report-ubuntu-2gb.json')
    ap.add_argument('--samples', type=int, default=100)
    ap.add_argument('--revision', default='unknown')
    a = ap.parse_args()
    if a.samples < 20:
        ap.error('samples must be >=20')
    binary = str(Path(a.binary).resolve())
    artifacts = Path(a.artifacts).resolve()
    output = Path(a.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    registry = artifacts/'registry.json'
    registry.unlink(missing_ok=True)
    names = ['hello', 'json_api', 'webhook', 'cpu', 'memory', 'large-response', 'trap', 'infinite_loop', 'oom', 'hello2']
    for name in names:
        artifact = {'hello2':'hello', 'cpu':'workload', 'memory':'workload', 'large-response':'workload', 'trap':'workload'}.get(name, name)
        manifest = artifacts/f'{name}.toml'
        manifest.write_text(f'name = "{name}"\nabi = "wednes:function@0.1.0"\nartifact = "{artifact}.wasm"\n[runtime]\nmemory_mb = 32\ntimeout_ms = 250\nmax_concurrency = 16\n')
        subprocess.run([binary,'deploy','--manifest',str(manifest),'--registry',str(registry)], check=True, stdout=subprocess.DEVNULL)
    before = proc_values('/proc/meminfo')
    vm_before = proc_values('/proc/vmstat')
    report = {'revision': a.revision, 'host': {'platform':platform.platform(), 'os_release':Path('/etc/os-release').read_text(), 'cpu':Path('/proc/cpuinfo').read_text().split('model name')[1].split('\n')[0] if 'model name' in Path('/proc/cpuinfo').read_text() else platform.machine(), 'cpus':os.cpu_count(), 'mem_total_kib':before['MemTotal']}, 'config': {'samples':a.samples,'concurrency':16,'memory_mb':32,'timeout_ms':250,'registered_functions':len(names)}, 'sha256': {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(binary), *artifacts.glob('*.wasm')]}, 'cases':{}, 'crash_count':0}
    log = open(output.with_suffix('.log'), 'w')
    process = None
    def start():
        t = time.perf_counter()
        p = subprocess.Popen([binary,'run','--registry',str(registry),'--listen','127.0.0.1:18081','--global-concurrency','16'], stdout=log, stderr=log)
        while time.perf_counter()-t < 60:
            if p.poll() is not None:
                raise RuntimeError('daemon exited before readiness')
            try:
                with socket.create_connection(('127.0.0.1',18081), timeout=.05):
                    return p, (time.perf_counter()-t)*1000
            except OSError:
                time.sleep(.002)
        p.terminate(); p.wait()
        raise RuntimeError('startup timeout')
    def call(name, expected=200, barrier=None):
        if barrier:
            barrier.wait(timeout=10)
        t = time.perf_counter()
        request = urllib.request.Request('http://127.0.0.1:18081/fn/'+name,data=b'{"hello":1}',method='POST')
        try:
            with urllib.request.urlopen(request,timeout=5) as response:
                status, body = response.status, response.read()
        except urllib.error.HTTPError as response:
            status, body = response.code, response.read()
        assert status == expected, (name,status,body[:200])
        if name == 'json_api':
            assert json.loads(body)['echo']['hello'] == 1
        if name == 'webhook':
            assert json.loads(body)['received'] is True
        return (time.perf_counter()-t)*1000
    try:
        process, startup = start()
        report['startup_latency_ms'] = startup
        report['idle_rss_kib'] = proc_values(f'/proc/{process.pid}/status')['VmRSS']
        report['ten_registered_idle_rss_kib'] = report['idle_rss_kib']
        cold = [call('hello')]
        for name in ['hello','json_api','webhook','cpu','memory','large-response']:
            path = {'cpu':'cpu/cpu','memory':'memory/memory','large-response':'large-response/large-response'}.get(name,name)
            values = [call(path) for _ in range(a.samples)]
            report['cases'][name] = {'count':len(values),**percentiles(values),'raw_ms':values}
        report['cases']['warm-start'] = report['cases']['hello']
        for name in ['infinite_loop','oom','trap/trap']:
            values = [call(name,500) for _ in range(3)]
            assert process.poll() is None
            call('hello')
            report['cases'][name] = {'count':len(values),**percentiles(values),'same_pid_survived':process.pid}
        malformed = artifacts/'bad.wasm'
        malformed.write_bytes(b'not wasm')
        bad_manifest = artifacts/'bad.toml'
        bad_manifest.write_text('name="bad"\nabi="wednes:function@0.1.0"\nartifact="bad.wasm"\n')
        prior = registry.read_bytes()
        bad = subprocess.run([binary,'deploy','--manifest',str(bad_manifest),'--registry',str(registry)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        assert bad.returncode != 0 and registry.read_bytes() == prior
        call('hello')
        report['malformed_rejected'] = True
        barrier = threading.Barrier(16)
        with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
            values = list(pool.map(lambda _:call('hello',barrier=barrier),range(16)))
        report['cases']['concurrency'] = {'clients':16,'successful':len(values),**percentiles(values)}
        report['peak_rss_kib'] = proc_values(f'/proc/{process.pid}/status')['VmHWM']
        report['post_suite_rss_kib'] = proc_values(f'/proc/{process.pid}/status')['VmRSS']
        assert process.poll() is None
        process.terminate(); process.wait(); process = None
        startups = [startup]
        for _ in range(19):
            process, elapsed = start()
            startups.append(elapsed)
            cold.append(call('hello'))
            process.terminate(); process.wait(); process = None
        report['cases']['cold-start'] = {'count':len(cold),**percentiles(cold),'raw_ms':cold,'definition':'first HTTP request after fresh daemon ready; startup compilation excluded'}
        report['startup_samples_ms'] = startups
        after = proc_values('/proc/meminfo')
        vm_after = proc_values('/proc/vmstat')
        report['swap'] = {'before_kib':before['SwapTotal']-before['SwapFree'],'after_kib':after['SwapTotal']-after['SwapFree'],'pswpin_delta':vm_after['pswpin']-vm_before['pswpin'],'pswpout_delta':vm_after['pswpout']-vm_before['pswpout']}
        report['verified'] = True
    except Exception as error:
        report['verified'] = False
        report['error'] = repr(error)
        if process is not None and process.poll() is not None:
            report['crash_count'] += 1
        raise
    finally:
        if process is not None and process.poll() is None:
            process.terminate(); process.wait()
        output.write_text(json.dumps(report,indent=2)+'\n')
        log.close()
    lines = ['# Ubuntu 2 GB benchmark','',f'Revision: `{a.revision}`. Measured on `{report["host"]["platform"]}`.', f'RAM: {before["MemTotal"]} KiB; CPUs: {os.cpu_count()}; 10 functions; 32 MiB guest memory; 250 ms timeout.', '', '| Metric | Measured | Target |','|---|---:|---:|',f'| Idle RSS (10 functions) | {report["idle_rss_kib"]/1024:.2f} MiB | 64 MiB (10 functions: 150 MiB) |', f'| Startup to listener | {startup:.2f} ms | <500 ms |', f'| Peak RSS | {report["peak_rss_kib"]/1024:.2f} MiB | measured |', f'| Swap used | {report["swap"]["after_kib"]} KiB | 0 |',f'| Guest-caused crashes | {report["crash_count"]} | 0 |','','| Case | p50 ms | p95 ms | p99 ms |','|---|---:|---:|---:|']
    for name, values in report['cases'].items():
        lines.append(f'| {name} | {values["p50_ms"]:.3f} | {values["p95_ms"]:.3f} | {values["p99_ms"]:.3f} |')
    lines += ['', 'Cold: first request on 20 fresh daemon processes. Warm: fresh guest instance, same compiled in-memory component. Python loopback HTTP latency includes client overhead. 16 barrier-synchronized clients do not prove 16 guest executions overlap. Startup recompiles source WASM; persistent cross-process cache is NOT implemented. No results inferred for unsupported features.', '', 'Reproduce: build components using `bash benchmarks/build.sh`, copy binary, components and this harness to Ubuntu; run `python3 benchmarks/run.py --binary ./wednesd --artifacts ./artifacts --output benchmarks/report-ubuntu-2gb.json --revision <git-sha>`. JSON contains raw latency samples, artifact hashes, hardware and configuration. Timeout/OOM/trap each checked three times, followed by success on same PID. Malformed deploy must leave registry byte-identical.', '']
    output.with_name('REPORT.md').write_text('\n'.join(lines))
    print(json.dumps({'verified':True,'idle_rss_kib':report['idle_rss_kib'],'startup_ms':startup,'crash_count':report['crash_count']}))


if __name__ == '__main__':
    main()
