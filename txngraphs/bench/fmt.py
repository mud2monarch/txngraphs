#!/usr/bin/env python3
import re
from statistics import mean, stdev

def parse_results(filename):
    with open(filename, "r") as f:
        text = f.read()

    sections = re.split(r"===\s*(\d+K blocks, \d+ depth)\s*===", text)
    results = {}
    for i in range(1, len(sections), 2):
        label = sections[i].replace(", 1 depth", "").strip()
        runs_text = sections[i+1]

        reals = [float(x) for x in re.findall(r"real\s+([\d.]+)", runs_text)]
        users = [float(x) for x in re.findall(r"user\s+([\d.]+)", runs_text)]
        syss  = [float(x) for x in re.findall(r"sys\s+([\d.]+)", runs_text)]

        # drop warm-up for 10K
        if label.startswith("10K") and reals:
            reals, users, syss = reals[1:], users[1:], syss[1:]

        real_mean = mean(reals) if reals else float("nan")
        real_sd   = stdev(reals) if len(reals) > 1 else 0.0  # sample stdev; use pstdev for population
        user_mean = mean(users) if users else float("nan")
        sys_mean  = mean(syss)  if syss  else float("nan")

        results[label] = {
            "real": real_mean,
            "user": user_mean,
            "sys":  sys_mean,
            "std dev": real_sd,   # only std dev of real
        }
    return results

def print_markdown_table(results):
    print("| Workload   | Real (s) | User (s) | Sys (s) | std dev (real) |")
    print("|------------|----------|----------|---------|---------|")
    for label, v in results.items():
        print(f"| {label:<10} | {v['real']:.2f}     | {v['user']:.2f}     | {v['sys']:.2f}    | {v['std dev']:.2f}   |")

if __name__ == "__main__":
    results = parse_results("bench/results.txt")
    print_markdown_table(results)
