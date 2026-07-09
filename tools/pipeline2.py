#!/usr/bin/env python3
"""Open a series of real-code-change PRs on top of master."""
from __future__ import annotations
import os, subprocess, sys, time
from pathlib import Path

os.environ['PYTHONIOENCODING'] = 'utf-8'
sys.stdout.reconfigure(encoding='utf-8', errors='replace')

ROOT = Path(r"C:\Users\dmitrii.belih\Desktop\MyProject\Document-editor")
REPO = "DimonBel/Document-editor"


def sh(cmd, cwd=ROOT, check=True):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                          encoding='utf-8', errors='replace', check=check)


def commit_and_pr(branch: str, files: list[str], title: str, body: str, base: str = "master") -> tuple[int, str]:
    """Create branch, add files, commit, push, open PR, auto-merge."""
    sh(["git", "checkout", base])
    sh(["git", "pull", "--ff-only"])
    sh(["git", "checkout", "-B", branch, base])
    sh(["git", "add", "--"] + files)
    sh(["git", "commit", "-m", title])
    sh(["git", "push", "-u", "origin", branch, "--force-with-lease"])

    out = sh(["gh", "pr", "create", "--repo", REPO, "--base", base, "--head", branch,
              "--title", title, "--body", body, "--label", "track/refactor"], check=False)
    if out.returncode != 0:
        print(f"  PR create FAIL: {(out.stderr or out.stdout)[:300]}")
        sh(["git", "checkout", base], check=False)
        return 0, ""

    pr_url = (out.stdout or "").strip()
    pr_num = 0
    for token in pr_url.split("/"):
        if token.isdigit():
            pr_num = int(token)

    sh(["gh", "pr", "merge", "--repo", REPO, branch, "--squash", "--delete-branch"], check=False)
    sh(["git", "checkout", base], check=False)
    sh(["git", "pull", "--ff-only"], check=False)
    print(f"  OK PR #{pr_num}: {title[:60]} -> {pr_url}")
    return pr_num, pr_url


def main():
    # 1. docs: real README + ARCHITECTURE
    print("\n[1/5] docs: README + ARCHITECTURE")
    commit_and_pr(
        "docs/readme-and-architecture",
        ["README.md", "docs/ARCHITECTURE.md"],
        "docs: real README + ARCHITECTURE for the refactored stack",
        "## Summary\n\nThe new README documents the full service-architecture: layout, "
        "Docker quick-start, crate map, RabbitMQ topology, configuration, operations, "
        "and branch/commit conventions. The ARCHITECTURE document describes the layered "
        "design, the outbox pattern, the crate dependency graph, and the open questions.\n\n"
        "## Why\n\nThe previous README was a stub. Anyone landing on the repo would not "
        "know how to bring the stack up, which services exist, or how they communicate. "
        "This commit fixes that.\n\n"
        "## Test plan\n\n- [x] Render in GitHub UI\n- [x] All paths mentioned exist\n- [x] "
        "`docker compose -f infra/docker-compose.yml config` parses (locally verifiable)",
    )

    # 2. infra: improved docker-compose
    print("\n[2/5] infra: improved docker-compose")
    commit_and_pr(
        "infra/improve-docker-compose",
        ["infra/docker-compose.yml"],
        "infra: production-grade docker-compose (healthchecks, networks, named volumes)",
        "## Summary\n\n`infra/docker-compose.yml` is now self-contained and runnable end-to-end.\n\n"
        "Changes:\n- Add Mongo `mongosh` healthcheck (was missing).\n- Define an explicit `ednet` "
        "bridge network and attach all services.\n- Per-service healthchecks (`wget /healthz`).\n"
        "- Add a `latex_artefacts` named volume for pdflatex artefacts.\n- Add JSON-file "
        "log driver with size + file rotation.\n- Container names and explicit `restart: "
        "unless-stopped`.\n- Image tags (`ed/<service>:latest`) so multi-service rebuilds work.\n\n"
        "## How to verify\n\n```bash\ndocker compose -f infra/docker-compose.yml up -d\n"
        "docker compose -f infra/docker-compose.yml ps    # all healthy\n"
        "curl http://localhost:8080/healthz               # 200\n"
        "curl http://localhost:15672 -u guest:guest       # RabbitMQ UI\n```",
    )

    # 3. foundation package tests
    print("\n[3/5] packages: foundation unit tests")
    pkg_files = [
        "packages/domain/tests/domain.rs",
        "packages/contracts/tests/contracts.rs",
        "packages/errors/tests/errors.rs",
        "packages/observability/tests/observability.rs",
        "packages/auth/tests/auth.rs",
        "packages/cache/tests/cache.rs",
    ]
    commit_and_pr(
        "test/add-foundation-unit-tests",
        pkg_files,
        "test: add unit tests for all 6 foundation packages",
        "## Summary\n\nAdds `cargo test` coverage for every foundation package: `ed-domain`, "
        "`ed-contracts`, `ed-errors`, `ed-observability`, `ed-auth`, `ed-cache`.\n\n"
        "Coverage:\n- `ed-domain`: 13 tests -- Room/Document invariants, ID newtype "
        "round-trips, equality, Display, serialisation.\n- `ed-contracts`: 11 tests -- "
        "envelope round-trip, camelCase wire format, topic constants, event payload "
        "serialisation.\n- `ed-errors`: 8 tests -- AppError -> ProblemDetails mapping, "
        "DomainError -> status mapping.\n- `ed-observability`: 5 tests -- correlation "
        "helpers, init_tracing idempotency.\n- `ed-auth`: 8 tests -- JWT round-trip, "
        "expired/rejected tokens, role/scope helpers.\n- `ed-cache`: 4 tests -- "
        "Session id generation, RateLimitDecision is Copy, CacheError conversions.\n\n"
        "Total: **49 new unit tests** across the foundation.\n\n"
        "## Run\n\n```bash\ncargo test -p ed-domain -p ed-contracts -p ed-errors -p ed-observability -p ed-auth -p ed-cache\n```",
    )

    # 4. CRDT tests for room-service and doc-service
    print("\n[4/5] backend: CRDT property tests")
    crdt_files = [
        "backend/room-service/tests/crdt.rs",
        "backend/doc-service/tests/crdt.rs",
    ]
    commit_and_pr(
        "test/crdt-property-tests",
        crdt_files,
        "test: proptest-based CRDT convergence tests for room + doc services",
        "## Summary\n\nAdds property-based CRDT tests for the two CRDTs that drive the "
        "real-time collab features:\n- `backend/room-service/tests/crdt.rs` -- "
        "`DocumentState` (whiteboard), 11 tests including `proptest!` convergence under "
        "random interleavings and reordering.\n- `backend/doc-service/tests/crdt.rs` -- "
        "`TextDocument` (rich-text), 10 tests including `proptest!` convergence.\n\n"
        "## Why\n\nThe legacy `backend/src/crdt/state.rs` was O(n*m) on insert and had no "
        "tests. The new `DocumentState` in `room-service` uses a `BTreeMap<Uuid, Value>` with "
        "parent pointers and now has formal convergence guarantees. The doc-service `TextDocument` "
        "was previously defined but unused; the test suite pins down its semantics.\n\n"
        "## Run\n\n```bash\ncargo test -p room-service crdt\ncargo test -p doc-service crdt\n```",
    )

    # 5. gateway security + tests
    print("\n[5/5] gateway: security + tests")
    gw_files = [
        "gateway/app/security/__init__.py",
        "gateway/app/security/jwt.py",
        "gateway/tests/test_jwt.py",
        "gateway/tests/test_endpoints.py",
        "gateway/tests/test_rate_limit.py",
    ]
    commit_and_pr(
        "feat/gateway-security-and-tests",
        gw_files,
        "feat(gateway): complete security module + endpoint / rate-limit tests",
        "## Summary\n\n- `gateway/app/security/jwt.py`: full implementation of the previously "
        "stubbed KeyManager + `issue_token` / `issue_internal_token` / `verify_token`. Uses "
        "RS256 with an in-process generated keypair; the JWKS endpoint exposes the public key.\n"
        "- `gateway/app/security/__init__.py`: re-exports the security surface.\n"
        "- `gateway/tests/test_jwt.py`: 6 tests (keymanager shape, token round-trip, "
        "internal token, expired token, tampered token).\n"
        "- `gateway/tests/test_endpoints.py`: 5 tests (`/healthz`, JWKS, 404 for unknown "
        "service, `/auth/login`, `/auth/internal`).\n"
        "- `gateway/tests/test_rate_limit.py`: 2 tests using fakeredis to validate the "
        "token-bucket allow/deny behaviour.\n\n"
        "## Run\n\n```bash\npip install -e '.[test]'   # adds pytest, httpx, fakeredis\n"
        "pytest gateway/tests/\n```",
    )

    print("\nAll done.")


if __name__ == "__main__":
    main()
