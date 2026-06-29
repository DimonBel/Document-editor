# Bot Setup — Document Editor (no API keys required)

All workflows below run on `GITHUB_TOKEN` alone — no secrets, no paid API keys, no accounts to create. Config files are also free / free-tier.

---

## Workflows already in this repo

| Workflow file           | What it does                                        | Trigger             |
|-------------------------|-----------------------------------------------------|---------------------|
| `reviewdog.yml`         | Runs `cargo clippy` (backend); ESLint hook reserved | PR open/sync + push master |
| `semgrep.yml`           | Static analysis (default + security + secrets)      | PR + push master    |
| `release-drafter.yml`   | Auto-drafts changelog from merged PR labels         | Push master         |
| `pr-failure-comment.yml`| Posts/updates a PR failure comment on any failed check | workflow_run (auto) |
| `pr-failure-issue.yml`  | Opens one auto-maintained `ci-failure` issue per outage | workflow_run (auto) |
| `review-findings.yml`   | Opens a tracker issue whenever a review bot (GHAS, CodeRabbit, Greptile, Qodo) posts a finding on a PR | PR review/comment (auto) |
| `sonar-community.yml`   | Self-hosted SonarQube Community Build               | **Manual only** (`workflow_dispatch`) |

All auto-run workflows need only `GITHUB_TOKEN`, which GitHub provides automatically. No setup required.

> **Why is sonar-community manual?** The bundled scanner-cli in `sonarsource/sonarqube-scan-action@v1` is Java 11, and the upstream `sonarqube:community` image is now SonarQube 12.x (Java 17 classes). The workflow now pins `sonarsource/sonarqube-scan-action@v4` with `scannerVersion: 6.2.1.4610` (Java 17) so the manual scan completes, but it remains `workflow_dispatch`-only to avoid pulling the heavy SonarQube image into every PR.

---

## Free-tier GitHub Apps (manual install — one click each, no API keys)

These need you to click "Install" on each provider's site and grant access to `DimonBel/Document-editor`. Free tiers, no API key from you:

| Bot              | Install URL                              | Notes                                    |
|------------------|------------------------------------------|------------------------------------------|
| CodeRabbit       | https://github.com/apps/coderabbitai     | Free for OSS / public repos; reads `.coderabbit.yaml` |
| Greptile         | https://github.com/apps/greptile         | Free tier; reads `.greptile/greptile.json` |
| Qodo Merge       | https://github.com/apps/qodo-merge       | Replaces the old PR-Agent; free tier     |
| Release Drafter  | already wired in workflow (no App)       | Uses GitHub labels                       |

After installing, the apps pick up their config files on the next PR.

---

## Bots deliberately NOT installed (require your own API key)

These were considered but removed because they need a paid API key you must obtain and store as a GitHub secret:

- Claude Code Review (Anthropic API key)
- OpenAI Codex Review (OpenAI key)
- Snyk (Snyk token)
- SonarCloud (Sonar token) — note: SonarCloud is a paid-token product for private repos; the Community Build above is the free local-container alternative
- AI Release Notes (OpenAI key)
- Cursor Bugbot, Macroscope, Sourcegraph Amp, Kodus — paid SaaS tiers

If you ever get keys for any of these, ask me to re-add the workflows.

---

## Activate

```bash
git add .
git commit -m "Add free-tier bot workflows and configs"
git push
```

That's it. Every new PR gets `cargo clippy` + `semgrep` checks, plus any GitHub Apps you installed. SonarQube is opt-in via the Actions tab.
