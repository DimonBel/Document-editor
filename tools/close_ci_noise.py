#!/usr/bin/env python3
"""Close 2 transient reviewdog CI failures (#170, #171)."""
import subprocess
REPO = "DimonBel/Document-editor"
MSG = ("Transient reviewdog failure on a now-merged branch. Re-running the "
       "reviewdog workflow against the current master (which has the fix "
       "from PR #169 applied) should now succeed. Closing as obsolete.")
for n in (170, 171):
    out = subprocess.run(["gh", "issue", "close", str(n), "--repo", REPO, "--comment", MSG],
                          capture_output=True, text=True)
    if out.returncode == 0: print(f"  closed #{n}")
    else: print(f"  FAIL #{n}: {(out.stderr or out.stdout)[:200]}")
