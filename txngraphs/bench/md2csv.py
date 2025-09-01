# ATTN @user
# Run script by passing the markdown filename and csv filename.
# E.g.: `uv run md2csv.py results.md results.csv`

#!/usr/bin/env python3
# md2csv.py
import csv, re, sys, unicodedata

CHUNK_RE   = re.compile(r"^\s*chunk_size:\s*([\d_]+)\s*$", re.IGNORECASE)
THREADS_RE = re.compile(r"^\s*rayon_threads:\s*([\d_]+)\s*$", re.IGNORECASE)

def to_float(s: str) -> float:
    s = unicodedata.normalize("NFKC", s).replace("\u00A0", " ").strip()
    s = s.replace(",", "")
    return float(s)

def parse_workload_label(label: str):
    # e.g. "40K blocks, 2 depth" or "10K blocks"
    m_blocks = re.search(r"(\d+)\s*K\s*blocks", label, re.IGNORECASE)
    blocks = int(m_blocks.group(1)) * 1000 if m_blocks else None
    m_depth = re.search(r"(\d+)\s*depth", label, re.IGNORECASE)
    depth = int(m_depth.group(1)) if m_depth else 1
    return blocks, depth

def is_separator_row(cells):
    # a row of dashes/colons typical of markdown separators
    return all(set(c.strip()) <= set("-:") and c.strip() for c in cells)

def md_to_csv(md_path: str, csv_path: str):
    rows = []
    current_chunk = None
    current_threads = None

    with open(md_path, "r", encoding="utf-8") as f:
        for raw in f:
            line = unicodedata.normalize("NFKC", raw).replace("\u00A0", " ").strip()

            m = CHUNK_RE.match(line)
            if m:
                current_chunk = int(m.group(1).replace("_", ""))
                current_threads = None  # reset so we don't leak prior value
                continue

            n = THREADS_RE.match(line)
            if n:
                current_threads = int(n.group(1).replace("_", ""))
                continue

            # Candidate table row
            if line.startswith("|") and line.count("|") >= 5:
                cells = [c.strip() for c in line.split("|")[1:-1]]  # drop leading/trailing empties
                if is_separator_row(cells):
                    continue
                if not cells:
                    continue
                if cells[0].lower().startswith("workload"):
                    continue  # header row

                # Support 4-col (old) or 5-col (new with std dev)
                if len(cells) == 4:
                    workload, real, user, sysv = cells
                    stddev = None
                elif len(cells) >= 5:
                    workload, real, user, sysv, stddev = cells[:5]
                else:
                    continue

                if current_chunk is None or current_threads is None:
                    continue  # require context lines before rows

                try:
                    r = to_float(real)
                    u = to_float(user)
                    s = to_float(sysv)
                    sd = to_float(stddev) if stddev not in (None, "", "nan") else 0.0
                except ValueError:
                    continue

                blocks, depth = parse_workload_label(workload)
                rows.append({
                    "chunk_size": current_chunk,
                    "rayon_threads": current_threads,
                    "workload": workload,
                    "blocks": blocks,
                    "depth": depth,
                    "real": r,
                    "user": u,
                    "sys": s,
                    "stddev_real": sd,
                })

    with open(csv_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=["chunk_size","rayon_threads","workload","blocks","depth","real","user","sys","stddev_real"]
        )
        writer.writeheader()
        writer.writerows(rows)

    print(f"Wrote {len(rows)} rows to {csv_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: md2csv.py results.md results.csv")
        sys.exit(1)
    md_to_csv(sys.argv[1], sys.argv[2])
