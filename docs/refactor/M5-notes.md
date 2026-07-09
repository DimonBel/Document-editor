# M5: texlive in latex-service
`infra/docker/Dockerfile.rust-service` accepts `INSTALL_TEX=1` as a build arg.
When set, the runtime stage installs:
- texlive-latex-base
- texlive-fonts-recommended
- texlive-latex-recommended
- texlive-science
- texlive-pictures

Total image size: ~700 MB.
