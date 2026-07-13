#!/usr/bin/env python3
"""Close the 4 stale noise issues (#16 #17 #159 #160) with comments."""
from __future__ import annotations
import os, subprocess, sys
os.environ['PYTHONIOENCODING'] = 'utf-8'
sys.stdout.reconfigure(encoding='utf-8', errors='replace')

REPO = "DimonBel/Document-editor"
ISSUES = [16, 17, 159, 160]

COMMENT = ("This issue is an auto-generated tracker from an earlier workflow run "
           "and is now obsolete (the underlying concern has been addressed in subsequent PRs "
           "or the workflow ran transiently during a refactor). Closing. Reopen if a "
           "concrete fix is still required.")

def sh(args):
    return subprocess.run(args, capture_output=True, text=True, encoding='utf-8', errors='replace')

for n in ISSUES:
    out = sh(["gh", "issue", "close", str(n), "--repo", REPO, "--comment", COMMENT])
    if out.returncode == 0:
        print(f"  closed #{n}")
    else:
        print(f"  FAIL #{n}: {(out.stderr or out.stdout)[:200]}")
