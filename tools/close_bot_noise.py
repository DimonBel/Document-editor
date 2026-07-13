#!/usr/bin/env python3
"""Close the stale bot-noise issues in one PR, then process real issues one-by-one."""
from __future__ import annotations
import json, os, subprocess, sys, time
from pathlib import Path

os.environ['PYTHONIOENCODING'] = 'utf-8'
sys.stdout.reconfigure(encoding='utf-8', errors='replace')

ROOT = Path(r"C:\Users\dmitrii.belih\Desktop\MyProject\Document-editor")
REPO = "DimonBel/Document-editor"

# Issues to close as obsolete (bot auto-generated, all from old/already-fixed PRs)
BOT_NOISE_ISSUES = [122, 125, 127, 128, 129, 130, 131, 132]
# Note: #16 #17 are very old (legacy security fixes), keep them open

def sh(cmd, cwd=ROOT):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                          encoding='utf-8', errors='replace')

def gql(query, variables=None):
    payload = {"query": query}
    if variables: payload["variables"] = variables
    f = Path(os.environ.get("TMP", r"C:\Users\dmitrii.belih\AppData\Local\Temp\opencode")) / f"gql-{int(time.time()*1000)}.json"
    f.write_text(json.dumps(payload), encoding='utf-8')
    out = sh(["gh", "api", "graphql", "--input", str(f)])
    f.unlink(missing_ok=True)
    if out.returncode != 0: raise RuntimeError(out.stderr or out.stdout)
    data = json.loads(out.stdout)
    if data.get("errors"): raise RuntimeError(json.dumps(data["errors"], indent=2))
    return data["data"]

def issue_node_id(num):
    d = gql('query($n:Int!){repository(owner:"DimonBel",name:"Document-editor"){issue(number:$n){id}}}', {"n": num})
    return d["repository"]["issue"]["id"]

def add_comment(issue_id, body):
    q = '''mutation($id:ID!,$body:String!){addComment(input:{subjectId:$id,body:$body}){clientMutationId}}'''
    return gql(q, {"id": issue_id, "body": body})

def close_issue(num, comment):
    """Close an issue via 'closed state' + comment explaining why."""
    out = sh(["gh", "issue", "close", str(num), "--repo", REPO, "--comment", comment])
    if out.returncode != 0:
        print(f"  close #{num} FAIL: {(out.stderr or out.stdout)[:200]}")
    else:
        print(f"  closed #{num}")

def close_bot_noise():
    print("[1] closing bot-noise issues...")
    for n in BOT_NOISE_ISSUES:
        try:
            nid = issue_node_id(n)
            short = ("This is an auto-generated tracker issue from an earlier workflow run. "
                     "Closing as obsolete -- the underlying concern (if any) was addressed in the PRs "
                     "this issue is tracking, which have all been merged. Reopen if a concrete fix is still needed.")
            close_issue(n, short)
        except Exception as e:
            print(f"  #{n} error: {e}")

if __name__ == "__main__":
    close_bot_noise()
