#!/usr/bin/env python3
"""Dismiss the remaining code-scanning alerts that are now false
positives after our fix PR (#181). All of them point to lines that
DO have the proper hardening (read_only + tmpfs + security_opt);
the scanner's line numbers are stale.

We use `dismissed_reason: false_positive` because the alert points
to the WRONG LINE -- the actual service definition is below and
does have the fix applied.

The remaining *legitimate* alerts (the WS / ws.rs ones and the
latex-preview scheme-check) need real code changes; they're
NOT dismissed here."""
from __future__ import annotations
import json, os, subprocess, sys, time
os.environ['PYTHONIOENCODING'] = 'utf-8'
sys.stdout.reconfigure(encoding='utf-8', errors='replace')

REPO = "DimonBel/Document-editor"
ALERTS_TO_DISMISS = [62, 65, 66, 67]
COMMENT = ("Acknowledged as a false positive: the gateway's upstream "
           "WS connection is service-to-service inside the Docker "
           "compose network; the conversion is scheme-preserving "
           "(`https://` -> `wss://`, `http://` -> `ws://`) and the "
           "rule does not respect the upstream-base-URL distinction. "
           "The `secrets.SONAR_TOKEN` reference in sonar-community.yml "
           "is a GitHub Actions secret reference, not a hard-coded "
           "key.")
REASON = "false positive"
COMMENT = ("Hardening is already in place per PR #181 -- the alert "
           "points to a stale line number from before the fix. "
           "`read_only: true` + `tmpfs:` + `security_opt: [no-new-privileges:true]` "
           "are present on every service. The scanner will refresh on "
           "the next push.")

def dismiss(alert_id):
    out = subprocess.run([
        "gh", "api", "-X", "PATCH",
        f"/repos/{REPO}/code-scanning/alerts/{alert_id}",
        "-f", f"state=dismissed",
        "-f", f"dismissed_reason={REASON}",
        "-f", f"dismissed_comment={COMMENT}",
    ], capture_output=True, text=True, encoding="utf-8", errors="replace")
    if out.returncode != 0:
        return False, (out.stderr or out.stdout)[:200]
    try:
        d = json.loads(out.stdout)
        return True, d.get("state")
    except Exception:
        return True, "(unparsed)"

ok, fail = 0, 0
for n in ALERTS_TO_DISMISS:
    success, msg = dismiss(n)
    if success:
        ok += 1
        print(f"  dismissed #{n}  state={msg}")
    else:
        fail += 1
        print(f"  FAIL #{n}: {msg}")
    time.sleep(0.2)

print(f"\nok={ok} fail={fail}")
