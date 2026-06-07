# Track sync hub — infrastructure as code

This directory defines **workspace-level** infrastructure: how to deploy and configure a Track sync hub. It is intentionally **separate** from project directories (issue tracking as code).

| Layer | Location | Version control |
|-------|----------|-----------------|
| Hub infra | `infra/workspaces/<slug>/` | This repo or dedicated infra repo |
| Project schema + work | `~/projects/<key>/` | Per-project git repos |
| Client credentials | `~/.config/track/config.json` | Never committed |

## Layout

```
infra/
├── README.md
└── workspaces/
    ├── personal/                 # example: single-operator + agents
    │   ├── hub.yaml              # workspace manifest (declarative config)
    │   └── deploy/
    │       ├── docker-compose.yml
    │       ├── .env.example
    │       └── Caddyfile         # optional TLS reverse proxy
    └── lab/                      # example: shared lab environment
        ├── hub.yaml
        └── deploy/
            ├── docker-compose.yml
            └── .env.example
```

## Quick start (personal workspace)

```bash
cd infra/workspaces/personal/deploy
cp .env.example .env
# Edit .env: POSTGRES_PASSWORD, TRACK_HUB_TOKEN_SECRET, etc.

docker compose up -d

# Register client credentials (on your machine, not in git)
track auth login \
  --workspace personal \
  --hub-url https://track-personal.example.com \
  --token "$TRACK_PAT"

# In a project directory
track push   # project track.yaml must set workspace: personal
```

## hub.yaml vs deploy/

- **`hub.yaml`** — Declarative workspace policy: slug, retention, auth rules, feature flags. Safe to commit. Consumed by hub on startup and by `track workspace validate`. Use `apiVersion: track.afofo.io/v1` at the top of the manifest.
- **`deploy/`** — Runtime deployment manifests (Compose, later Terraform/Kubernetes). Environment-specific values go in `.env` (gitignored).

## Adding a workspace

1. Copy `workspaces/personal/` to `workspaces/<new-slug>/`
2. Edit `hub.yaml`: `workspace.slug`, URLs, retention
3. Edit `deploy/.env.example` and deploy
4. Point client `track auth login --workspace <new-slug>`

Projects associate via `workspace: <slug>` in their `track.yaml`; they do not embed hub URLs or secrets.

## Production notes

- Use managed PostgreSQL or persistent volumes for `postgres_data`
- Terminate TLS at Caddy/Traefik or a load balancer
- Rotate `TRACK_HUB_TOKEN_SECRET` and issue per-actor tokens (human PAT, `agent:*`, CI service token)
- Set `event_retention_days` per compliance needs
- Hub holds the **full issue/effort index**; clients materialize lazily (see PRD §8.1)

See [docs/PRD.md](../docs/PRD.md) for product intent; [docs/SRD.md](../docs/SRD.md) for technical design (SRD §3.1 describes lazy materialization).
