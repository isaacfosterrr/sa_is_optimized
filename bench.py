#!/usr/bin/env python3
import re, subprocess, sys, statistics as st
from collections import OrderedDict

binary = sys.argv[1] if len(sys.argv) > 1 else "./target/release/sa_linear"
runs = int(sys.argv[2]) if len(sys.argv) > 2 else 5
extra = sys.argv[3:]   # optional extra perf args, e.g. -e cache-misses,cache-references

p = subprocess.run(["perf", "stat", "-r", str(runs), *extra, "--", binary],
                   capture_output=True, text=True)

pat = re.compile(r"^(.+?) depth=(\d+), n=\d+, time=([\d.]+)s")
per_label = OrderedDict()   # (name, depth) -> one value per run
perf_lines, in_perf = [], False

for line in p.stderr.splitlines():
    m = pat.match(line)
    if m:
        per_label.setdefault((m.group(1), int(m.group(2))), []).append(float(m.group(3)))
    elif "Performance counter stats" in line:
        in_perf = True
        perf_lines.append(line)
    elif in_perf:
        perf_lines.append(line)

def fmt(xs):
    sd = st.stdev(xs) if len(xs) > 1 else 0.0
    return f"{st.mean(xs):8.3f} {sd:7.3f} {min(xs):8.3f}"

print(f"{'phase':<22}{'depth':>5} {'mean s':>8} {'stdev':>7} {'min':>8}")
for (name, depth), xs in per_label.items():
    print(f"{name:<22}{depth:>5} {fmt(xs)}")
print()
print("\n".join(perf_lines) if perf_lines else p.stderr)
