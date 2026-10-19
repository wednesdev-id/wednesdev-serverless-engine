# systemd Service

For bare-metal and virtual machine deployments (such as Ubuntu/Debian cloud nodes), running `wednesd` as a supervised **systemd** service ensures automatic restarts, security sandboxing, and log forwarding to `journald`.

---

## Service Unit File

Create `/etc/systemd/system/wednesd.service`:

```ini
[Unit]
Description=Wednes Serverless Engine Function Daemon
Documentation=https://github.com/wednesdev-id/wednesdev-serverless-engine
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=wednesdev
Group=wednesdev
WorkingDirectory=/var/lib/wednes

ExecStart=/usr/local/bin/wednesd run \
  --registry /var/lib/wednes/registry.json \
  --listen 0.0.0.0:8080 \
  --global-concurrency 200 \
  --memory-budget-mb 1536 \
  --log-json

Restart=always
RestartSec=3s

# Resource Constraints
LimitNOFILE=65536
MemoryMax=1800M
CPUQuota=200%

# Linux Security Sandboxing
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/wednes
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX

[Install]
WantedBy=multi-user.target
```

---

## Installation & Lifecycle Management

### 1. Create Working Directory

```bash
sudo mkdir -p /var/lib/wednes
sudo chown -R wednesdev:wednesdev /var/lib/wednes
```

### 2. Copy Executable

```bash
sudo cp target/release/wednesd /usr/local/bin/wednesd
sudo chmod +x /usr/local/bin/wednesd
```

### 3. Enable and Start Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable wednesd
sudo systemctl start wednesd
```

### 4. Check Status and Logs

```bash
# Check service state
sudo systemctl status wednesd

# Follow live JSON logs via journalctl
sudo journalctl -u wednesd -f -o cat
```
