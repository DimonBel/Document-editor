#!/usr/bin/env python3
"""Close #141 #149 #165 with explanation comments."""
import subprocess
REPO = "DimonBel/Document-editor"
MSG = ("Closed by the merged PR. The commit-message keyword parser did not "
       "auto-close because multiple issue numbers were joined with commas; "
       "re-closing manually. Fix is in PR #164 (idempotency) for #141, "
       "PR #166 (outbox serialisation) for #149, and #165 was a transient "
       "reviewdog CI failure already addressed.")
for n in (141, 149, 165):
    out = subprocess.run(["gh", "issue", "close", str(n), "--repo", REPO, "--comment", MSG],
                          capture_output=True, text=True)
    if out.returncode == 0: print(f"  closed #{n}")
    else: print(f"  FAIL #{n}: {(out.stderr or out.stdout)[:200]}")
