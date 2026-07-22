#!/usr/bin/env python3
"""Add `security_opt: [no-new-privileges:true]` and (for stateful
services) `read_only: true` + `tmpfs:` mounts to every service in
`infra/docker-compose.yml`."""

import re
from pathlib import Path

P = Path(r"C:\Users\dmitrii.belih\Desktop\MyProject\Document-editor\infra\docker-compose.yml")
src = P.read_text(encoding="utf-8")

stateful_tmpfs = {
    "postgres": "      - /tmp\n      - /var/run/postgresql\n      - /dev/shm\n",
    "mongo":    "      - /tmp\n      - /dev/shm\n",
    "redis":    "      - /tmp\n",
    "rabbit":   "      - /tmp\n",
}

lines = src.split("\n")
out_lines = []
i = 0
n = len(lines)
current_service = None
current_service_start = -1

while i < n:
    line = lines[i]
    m = re.match(r"^  ([a-z][a-z0-9_-]*):\s*$", line)
    if m:
        name = m.group(1)
        if name in stateful_tmpfs or True:  # every name: line is a service
            current_service = name
            current_service_start = len(out_lines)

    # Check: this line ends the service block (next top-level key)
    if current_service and i > 0 and re.match(r"^  [a-z][a-z0-9_-]*:\s*$", line) and not re.match(r"^  name:\s*$", line) and line != lines[i]:
        # Just process at the end. For now we accumulate and post-process.
        pass

    out_lines.append(line)
    i += 1

# Now post-process: for each service block, inject security_opt
# and read_only+tmpfs.
def find_block(name):
    """Return (start, end_exclusive) line indices for the block whose
    first line is `  <name>:`."""
    for i, line in enumerate(out_lines):
        if re.match(rf"^  {re.escape(name)}:\s*$", line):
            # find the next top-level key or end of services: block
            j = i + 1
            while j < n:
                if re.match(r"^  [a-z][a-z0-9_-]*:\s*$", out_lines[j]):
                    break
                j += 1
            return i, j
    return None, None

def insert_after(start, end, target_line_re, new_lines):
    """Insert `new_lines` after the first line in `out_lines[start:end]`
    that matches `target_line_re`. If no match, append before end."""
    for k in range(start, end):
        if target_line_re.match(out_lines[k]):
            return k + 1, out_lines[:k+1] + new_lines + out_lines[k+1:]
    return end, out_lines[:end] + new_lines + out_lines[end:]

def insert_before(start, end, target_line_re, new_lines):
    """Insert `new_lines` before the first line in [start, end) that
    matches `target_line_re`. If no match, append at end."""
    for k in range(start, end):
        if target_line_re.match(out_lines[k]):
            return k, out_lines[:k] + new_lines + out_lines[k:]
    return end, out_lines[:end] + new_lines + out_lines[end:]

# Process backwards so insertions don't disturb indices we'll visit.
all_services = []
i = 0
while i < n:
    m = re.match(r"^  ([a-z][a-z0-9_-]*):\s*$", out_lines[i])
    if m:
        # find end
        j = i + 1
        while j < n:
            if re.match(r"^  [a-z][a-z0-9_-]*:\s*$", out_lines[j]):
                break
            j += 1
        all_services.append((i, j, m.group(1)))
        i = j
    else:
        i += 1

# Apply transforms backwards.
for (i, j, name) in reversed(all_services):
    block = "\n".join(out_lines[i:j])
    # 1) security_opt after image:
    if "security_opt:" not in block:
        # Find the image: line
        m2 = re.search(r"^[ \t]+image:[^\n]*$", block, re.MULTILINE)
        if m2:
            insert_at = i + m2.end()  # image line ends -- insert after
            new_line = "    security_opt: [no-new-privileges:true]"
            out_lines = out_lines[:insert_at] + [new_line] + out_lines[insert_at:]
            j += 1
        else:
            # No image; insert as the first scalar key after `name:`
            m2 = re.search(r"^[ \t]+name:[^\n]*$", block, re.MULTILINE)
            if m2:
                insert_at = i + m2.end()
                new_line = "    security_opt: [no-new-privileges:true]"
                out_lines = out_lines[:insert_at] + [new_line] + out_lines[insert_at:]
                j += 1

    # 2) read_only + tmpfs (for stateful services only)
    if name in stateful_tmpfs and "read_only:" not in block:
        block = "\n".join(out_lines[i:j])
        if "read_only:" not in block:
            # Insert just before `volumes:` (or before the
            # healthcheck/networks block if no volumes)
            m2 = re.search(r"^[ \t]+volumes:[ \t]*$", block, re.MULTILINE)
            insertion_lines = ["    read_only: true", "    tmpfs:"] + stateful_tmpfs[name].rstrip("\n").split("\n")
            if m2:
                # position absolute: i + m2.start()
                insert_at = i + m2.start()
                # split text
                out_lines = out_lines[:insert_at] + insertion_lines + out_lines[insert_at:]
                j += len(insertion_lines)
            else:
                # No volumes; insert before the first action key
                m3 = re.search(r"^[ \t]+(?:healthcheck|networks|logging|depends_on):[ \t]*$", block, re.MULTILINE)
                if m3:
                    insert_at = i + m3.start()
                    out_lines = out_lines[:insert_at] + insertion_lines + out_lines[insert_at:]
                    j += len(insertion_lines)
                else:
                    # Last resort: insert at end of block
                    out_lines = out_lines[:j] + insertion_lines + out_lines[j:]
                    j += len(insertion_lines)

P.write_text("\n".join(out_lines) + "\n", encoding="utf-8")
print("ok")
