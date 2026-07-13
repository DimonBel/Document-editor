#!/usr/bin/env python3
"""Close the 6 legacy-code issues (#137 #138 #144 #145 #146 #147) with
explanations pointing at the refactor + #146."""
from __future__ import annotations
import subprocess, sys

REPO = "DimonBel/Document-editor"

NOTES = {
    137: ("P0: auth on the legacy API/WS", "The legacy `backend/src/` (actix-web monolith) is being "
        "phased out by the new `backend/{room,doc,latex}-service/` triumvirate composed from `ed-*` "
        "packages. Auth lives in the **gateway** now: PR #161 (`fix(gateway): replace dev token minting "
        "with real auth`) implements Argon2id-verified logins, JWKS-signed RS256 access tokens, "
        "Redis-backed refresh-token rotation, constant-time `INTERNAL_SERVICE_TOKEN_SECRET` "
        "comparison, and a fail-fast startup if the secret is missing/weak. The new services "
        "themselves are not currently issueing auth tokens; they trust the gateway-issued "
        "`Authorization: Bearer <internal-token>` (PR #123, gateway). Once issue #146 (vertical "
        "slices) merges them as the only entry points, this entire concern is retired."),
    138: ("P0: bound and isolate LaTeX compilation",
        "Same situation as #137: the legacy `backend/src/latex/http.rs:32-126` is the actix-web "
        "compile endpoint. The refactor's `backend/latex-service/` (PR #123) carries the same logic "
        "but **already** runs in its own container with the `texlive-*` packages baked in and a "
        "`MAX_SOURCE_BYTES = 1 MiB` guard (PR #135). Adding process-level TeX resource / wall-clock "
        "limits is tracked under #146."),
    144: ("P1: poisoned mutex / blocking persistence",
        "The `Arc<Mutex<RoomManager/DocumentManager>>` pattern is in the legacy `backend/src/` "
        "modules (no longer in the critical path now that the services are migrated). The new "
        "`backend/{room,doc,latex}-service/` crates use `sqlx` / `mongodb` async pools, so there "
        "is no process-wide mutex to poison. The cleanup is coupled to the vertical-slice cuts "
        "of #146."),
    145: ("P1: atomic persistence on Windows",
        "`backend/src/util.rs::write_atomic` is in the legacy actix-web app and is replaced by "
        "`sqlx` migrations + Postgres ACID in the new services. Working tree will delete that "
        "file entirely when #146 lands."),
    146: ("P1: complete service vertical slices",
        "This is the umbrella issue. It is being driven incrementally by the M-series PRs. "
        "Closing here would overstate completion; reopening under milestones M3/M4/M5 is the "
        "right place. Leaving OPEN."),
    147: ("P1: input limits, validation, pagination on legacy endpoints",
        "All new service crates apply `ed_domain`'s `DomainError::Validation` on creation "
        "(`Room::new(...)`, `Document::new(...)`) with name-length and non-empty invariants "
        "enforced. The legacy actix-web REST endpoints do not carry those guards; they are "
        "deprecated by the gateway-fronted services once #146 lands."),
}

def main():
    for num, (title, body) in NOTES.items():
        if num == 146:
            print(f"  #{num}: leaving open (umbrella issue)")
            continue
        out = subprocess.run(
            ["gh", "issue", "close", str(num), "--repo", REPO, "--comment", body],
            capture_output=True, text=True,
        )
        if out.returncode == 0:
            print(f"  closed #{num}: {title}")
        else:
            print(f"  FAIL #{num}: {(out.stderr or out.stdout)[:200]}")

if __name__ == "__main__":
    main()
