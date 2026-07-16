#!/usr/bin/env python3
import sys
import time
import json
import argparse
from datetime import datetime


def parse_args():
    p = argparse.ArgumentParser(description="Replay JSONL GPS as timed stdout")
    p.add_argument("jsonl", help="Input JSONL with fields: ts, lat, lon, altitude_m, source")
    p.add_argument("--speed", type=float, default=1.0, help="Time scale (2.0 = 2x faster)")
    p.add_argument("--loop", action="store_true", help="Loop forever")
    p.add_argument("--delay", type=float, default=0.0, help="Seconds to wait before first sample")
    return p.parse_args()


def iso_to_epoch(s: str) -> float:
    if s.endswith('Z'):
        s = s[:-1] + "+00:00"
    return datetime.fromisoformat(s).timestamp()


def replay(path: str, speed: float, loop: bool, delay: float) -> None:
    while True:
        prev_t = None
        first = True
        with open(path, "r") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                except Exception:
                    continue
                t = iso_to_epoch(str(obj.get("ts")))
                if first:
                    if delay > 0:
                        time.sleep(delay)
                    first = False
                if prev_t is not None:
                    dt = max(0.0, (t - prev_t) / max(1e-6, speed))
                    time.sleep(dt)
                sys.stdout.write(json.dumps(obj) + "\n")
                sys.stdout.flush()
                prev_t = t
        if not loop:
            break


def main() -> None:
    a = parse_args()
    replay(a.jsonl, a.speed, a.loop, a.delay)


if __name__ == "__main__":
    main()
