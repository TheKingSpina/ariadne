#!/usr/bin/env python3
"""Preparazione documentale: piani L6 e oracoli candidati, non esecuzione Ariadne.

Scrive due JSONL nuovi nella directory fornita. Nessun modulo Ariadne è
importato, nessun log di run è generato e nessun codice di produzione è
eseguito. La procedura richiede revisione indipendente prima della firma.
"""

import argparse
import hashlib
import json
from pathlib import Path

MASK64 = (1 << 64) - 1
MASK128 = (1 << 128) - 1
MULTIPLIER = 0x2360ED051FC65DA44385DF649FCCF645


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def generate_plan(seed):
    counter = seed

    def splitmix():
        nonlocal counter
        counter = (counter + 0x9E3779B97F4A7C15) & MASK64
        z = counter
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
        return z ^ (z >> 31)

    initial, sequence = splitmix(), splitmix()
    increment = (sequence << 1) | 1
    state = ((initial + increment) * MULTIPLIER + increment) & MASK128

    def draw():
        nonlocal state
        state = (state * MULTIPLIER + increment) & MASK128
        rotation = state >> 122
        mixed = ((state >> 64) ^ state) & MASK64
        return ((mixed >> rotation) | (mixed << ((-rotation) & 63))) & MASK64

    rule = [i // 4 for i in range(12)]
    for i in range(11, 0, -1):
        j = draw() % (i + 1)
        rule[i], rule[j] = rule[j], rule[i]
    indices = [draw() % 12 for _ in range(80)]
    indices += list(range(12)) * 3
    indices += [draw() % 12 for _ in range(4)]
    tasks = [{"cue": f"cue:{i:02d}", "giusta": rule[i]} for i in indices]
    return {"rule": rule, "seed": seed, "tasks": tasks}, indices


def expected_sums(plan, indices):
    counts = [[0, 0, 0] for _ in range(12)]
    memory_sum = control_sum = 0
    for position, cue in enumerate(indices):
        correct = plan["rule"][cue]
        predicted = max(range(3), key=lambda value: (counts[cue][value], -value))
        if position >= 80:
            memory_sum += 1000 if predicted == correct else 0
            control_sum += 1000 if correct == 0 else 0
        counts[cue][correct] += 1
    return memory_sum, control_sum


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="directory nuova di output")
    args = parser.parse_args()
    args.output.mkdir(parents=False, exist_ok=False)
    plan_lines, oracle_lines = [], []
    for seed in range(1000, 1100):
        plan, indices = generate_plan(seed)
        plan_bytes = canonical(plan)
        memory, control = expected_sums(plan, indices)
        row = {
            "control_test_milli_sum": control,
            "memory_test_milli_sum": memory,
            "plan_sha256": hashlib.sha256(plan_bytes).hexdigest(),
            "seed": seed,
        }
        plan_lines.append(plan_bytes + b"\n")
        oracle_lines.append(canonical(row) + b"\n")
    for name, lines in (("piani.jsonl", plan_lines), ("oracoli.jsonl", oracle_lines)):
        data = b"".join(lines)
        with (args.output / name).open("xb") as file:
            file.write(data)
        print(f"{hashlib.sha256(data).hexdigest()}  {name}")


if __name__ == "__main__":
    main()
