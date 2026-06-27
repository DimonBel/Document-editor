# Bot Setup — Document Editor (no API keys required)

All workflows below run on `GITHUB_TOKEN` alone — no secrets, no paid API keys, no accounts to create. Config files are also free / free-tier.

---

## Workflows already in this repo (work automatically on PR)

| Workflow file           | What it does                                        | Trigger             |
|-------------------------|-----------------------------------------------------|---------------------|
| `reviewdog.yml`         | Posts ESLint + clippy lint findings as PR comments  | PR open/sync        |
| `semgrep.yml`           | Static analysis (default + security + secrets)      | PR + push master    |
| `release-drafter.yml`   | Auto-drafts changelog from merged PR labels         | Push master         |
| `sonar-community.yml`   | Self-hosted SonarQube Community Build               | PR + push master    |

All four need only `GITHUB_TOKEN`, which GitHub provides automatically. No setup required.

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
- SonarCloud (Sonar token)
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

That's it. Every new PR will get reviewdog + semgrep + sonar-community comments automatically, and the GitHub Apps you installed will chime in too.