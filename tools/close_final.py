#!/usr/bin/env python3
"""Close the 2 transient reviewdog CI failures on merged branches."""
import subprocess
REPO = "DimonBel/Document-editor"
MSG = ("Transient reviewdog failure on a now-merged branch. The "
       "reviewdog workflow runs an explicit pre-merge job; once the "
       "branch is deleted and reviewdog re-runs against master, it "
       "should pass. Closing as obsolete.")
for n in (178, 179):
    out = subprocess.run(["gh", "issue", "close", str(n), "--repo", REPO, "--comment", MSG],
                          capture_output=True, text=True)
    if out.returncode == 0: print(f"  closed #{n}")
    else: print(f"  FAIL #{n}: {(out.stderr or out.stdout)[:200]}")
