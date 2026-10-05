"""Run: python3 -m unittest discover -s benchmarks -p 'test_*.py'."""
import tempfile
import unittest
from pathlib import Path
from run import percentiles, proc_values


class BenchmarkTests(unittest.TestCase):
    def test_nearest_rank(self):
        self.assertEqual(percentiles(list(range(1,101))), {'p50_ms':50,'p95_ms':95,'p99_ms':99})
        self.assertEqual(percentiles([2]), {'p50_ms':2,'p95_ms':2,'p99_ms':2})

    def test_reject_invalid_samples(self):
        for samples in ([], [-1], [float('nan')], [float('inf')]):
            with self.assertRaises(ValueError):
                percentiles(samples)

    def test_proc_units_and_nonnumeric_fields(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'status'
            path.write_text('Name:\twednesd\nVmRSS:\t1024 kB\nVmSwap:\t0 kB\n')
            self.assertEqual(proc_values(path), {'VmRSS':1024,'VmSwap':0})


if __name__ == '__main__':
    unittest.main()
