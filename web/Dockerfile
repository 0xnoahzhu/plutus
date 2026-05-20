## Plutus web SSR — Node 24 + pnpm + tsx watch-less runtime.
##
## The web app has no build step (server.ts runs straight through tsx),
## so the image just installs deps and runs the server.
##
## Build from the `web/` directory:
##   podman build -t plutus-web:latest web/

FROM docker.io/library/node:24-trixie-slim

# Install pnpm via corepack. Pinning to the lockfile's required version
# is handled automatically by `corepack enable` + `pnpm install`.
RUN corepack enable

# Tini for clean signal handling under systemd / Quadlet. Node's default
# signal behavior under PID 1 is awkward; tini fixes that.
RUN apt-get update \
 && apt-get install -y --no-install-recommends tini ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Non-root runtime. node:24-trixie-slim already ships a `node` user
# (uid 1000); reuse it instead of inventing another.
WORKDIR /app
RUN chown -R node:node /app
USER node

# Install dependencies first so source changes don't bust the deps layer.
COPY --chown=node:node package.json pnpm-lock.yaml ./
# pnpm 11 refuses to run dependency postinstall scripts by default
# (esbuild ships a native binary via a postinstall download). The
# `--config.dangerouslyAllowAllBuilds=true` flag opts in for this image
# only — fine here because the lockfile pins exact versions.
RUN pnpm install --frozen-lockfile --config.dangerouslyAllowAllBuilds=true

# Now bring the app source. node_modules is excluded via .dockerignore.
COPY --chown=node:node . .

ENV NODE_ENV=production
ENV PORT=4100

EXPOSE 4100

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["pnpm", "start"]
