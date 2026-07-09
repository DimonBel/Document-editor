# M6: Cutover

1. Move legacy `docker-compose.yml` -> `legacy/docker-compose.yml.bak`
2. Promote `infra/docker-compose.yml` -> root as `docker-compose.yml`
3. Update `SETUP.md` and CI workflows to build the new workspaces
4. Open PR `refactor/backend-services -> master`
