# Issue #65 -- ❌ CI Failure — reviewdog on master (18b5c20)

**Status:** Done (PR auto-merged).

## What was done

<!-- ci-failure-tracker -->
<!-- ci-failure-tracker:run=29017185184 -->
## ❌ CI Failure Detected
| Field | Value |
|---|---|
| Workflow | `reviewdog` |
| Branch | `master` |
| Commit | `18b5c20` |
| Triggered by | @DimonBel |
| Event | `push` |
| Run started | 2026-07-09T12:09:12Z |
📋 [View full run logs](https://github.com/DimonBel/Document-editor/actions/runs/29017185184)
### Log excerpt
```
===== 0_clippy _ backend.txt =====
﻿2026-07-09T12:09:16.0218631Z Current runner version: '2.335.1'
2026-07-09T12:09:16.0252213Z ##[group]Runner Image Provisioner
2026-07-09T12:09:16.0253830Z Hosted Compute Agent
2026-07-09T12:09:16.0254918Z Version: 20260624.560
2026-07-09T12:09:16.0255947Z Commit: 925d229a51159bc391ae97e54a2dd1fe20af789d
2026-07-09T12:09:16.0257273Z Build Date: 2026-06-24T18:26:47Z
2026-07-09T12:09:16.0258459Z Worker ID: {b9f8a1c5-97b5-4f71-96b4-4b820dba9ee0}
2026-07-09T12:09:16.0259607Z Azure Region: eastus
2026-07-09T12:09:16.0260572Z ##[endgroup]
2026-07-09T12:09:16.0263151Z ##[group]Operating System
2026-07-09T12:09:16.0264258Z Ubuntu
2026-07-09T12:09:16.0265118Z 24.04.4
2026-07-09T12:09:16.0265953Z LTS
2026-07-09T12:09:16.0266846Z ##[endgroup]
2026-07-09T12:09:16.0267699Z ##[group]Runner Image
2026-07-09T12:09:16.0268701Z Image: ubuntu-24.04
2026-07-09T12:09:16.0269744Z Version: 20260705.232.1
2026-07-09T12:09:16.0271692Z Included Software: https://github.com/actions/runner-images/blob/ubuntu24/20260705.232/images/ubuntu/Ubuntu2404-Readme.md
2026-07-09T12:09:16.0274716Z Image Release: https://github.com/actions/runner-images/releases/tag/ubuntu24%2F20260705.232
2026-07-09T12:09:16.0276374Z ##[endgroup]
2026-07-09T12:09:16.0278456Z ##[group]GITHUB_TOKEN Permissions
2026-07-09T12:09:16.0281439Z Contents: read
2026-07-09T12:09:16.0282843Z Metadata: read
2026-07-09T12:09:16.0283840Z PullRequests: write
2026-07-09T12:09:16.0284834Z ##[endgroup]
2026-07-09T12:09:16.0287856Z Secret source: Actions
2026-07-09T12:09:16.0289057Z Prepare workflow directory
2026-07-09T12:09:16.0758230Z Prepare all required actions
2026-07-09T12:09:16.0814505Z Getting action download info
2026-07-09T12:09:16.3470880Z Download action repository 'actions/checkout@v4' (SHA:34e114876b0b11c390a56381ad16ebd13914f8d5)
2026-07-09T12:09:16.4195909Z Download action repository 'dtolnay/rust-toolchain@stable' (SHA:4be7066ada62dd38de10e7b70166bc74ed198c30)
2026-07-09T12:09:16.5538262Z Download action repository 'Swatinem/rust-cache@v2' (SHA:e18b497796c12c097a38f9edb9d0641fb99eee32)
2026-07-09T12:09:16.9425904Z Complete job name: clippy / backend
2026-07-09T12:09:17.0166901Z Node 20 is being deprecated. This workflow is running with Node 24 by default. If you need to temporarily use Node 20, you can set the ACTIONS_ALLOW_USE_UNSECURE_NODE_VERSION=true environment variable. For more information see: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/
2026-07-09T12:09:17.0176446Z ##[group]Run actions/checkout@v4
2026-07-09T12:09:17.0177396Z with:
2026-07-09T12:09:17.0177876Z   repository: DimonBel/Document-editor
2026-07-09T12:09:17.0181520Z   token: ***
2026-07-09T12:09:17.0181956Z   ssh-strict: true
2026-07-09T12:09:17.0182578Z   ssh-user: git
2026-07-09T12:09:17.0183038Z   persist-credentials: true
2026-07-09T12:09:17.0183539Z   clean: true
2026-07-09T12:09:17.0183983Z   sparse-checkout-cone-mode: true
2026-07-09T12:09:17.0184545Z   fetch-depth: 1
2026-07-09T12:09:17.0184984Z   fetch-tags: false
2026-07-09T12:09:17.0185418Z   show-progress: true
2026-07-09T12:09:17.0185879Z   lfs: false
2026-07-09T12:09:17.0186305Z   submodules: false
2026-07-09T12:09:17.0186754Z   set-safe-directory: true
2026-07-09T12:09:17.0187844Z ##[endgroup]
2026-07-09T12:09:17.1280330Z Syncing repository: DimonBel/Document-editor
2026-07-09T12:09:17.1285842Z ##[group]Getting Git version info
2026-07-09T12:09:17.1286685Z Working directory is '/home/runner/work/Document-editor/Document-editor'
2026-07-09T12:09:17.1287961Z [command]/usr/bin/git version
2026-07-09T12:09:17.1381695Z git version 2.54.0
2026-07-09T12:09:17.1437554Z ##[endgroup]
2026-07-09T12:09:17.1462139Z Temporarily overriding HOME='/home/runner/work/_temp/28a8c6f3-0181-4c86-b15f-33ee6f03a51f' before making global git config changes
2026-07-09T12:09:17.1464940Z Adding repository directory to the temporary git global config as a safe directory
2026-07-09T12:09:17.1466917Z [command]/usr/bin/git config --glo

```
---
_Auto-maintained by the CI Failure Issue workflow._
_A new failure on the same open issue updates this body in place._

## Files changed in the initial scaffolding commit

See commit `449281a` on branch `refactor/backend-services` (or `master` after merge).
