# Issue #61 -- [latex-service] Bake `texlive-latex-base` into the Dockerfile

**Status:** Done (PR auto-merged).

## What was done

Currently the runtime image lacks `pdflatex` (existing `backend/Dockerfile` does NOT install it). Fix: derive a variant that adds `texlive-latex-base`, `texlive-fonts-recommended`, `texlive-latex-recommended`, `texlive-science`, `texlive-pictures` to the runtime stage.

**Acceptance**
- `docker compose -f infra/docker-compose.yml up latex-service` then `curl -F source=@sample.tex http://gateway:8080/api/v1/latex-service/api/latex/compile` returns 200 + a valid PDF.
- Image < 800 MB.

## Files changed in the initial scaffolding commit

See commit `449281a` on branch `refactor/backend-services` (or `master` after merge).
