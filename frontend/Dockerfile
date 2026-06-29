# ── Builder stage ──────────────────────────────────────────────────────────
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install
COPY . .
RUN npm run build

# ── Runtime stage (nginx) ──────────────────────────────────────────────────
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
# Proxy API + WS to backend
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
# Run nginx as the unprivileged `nginx` user shipped by the official
# nginx:alpine image (UID 101). The default nginx config expects to
# run as this user and its PID file, logs, and the temp dirs under
# /var/cache/nginx /var/run are owned by it.
USER nginx
CMD ["nginx", "-g", "daemon off;"]
