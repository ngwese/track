# Air-gapped and offline component distribution

Track separates the **host** (`track-host`, native) from the **CLI component** (`track-cli.wasm`). In air-gapped environments you can vendor the component instead of fetching it at runtime.

## Default layout (networked)

The host resolves `track-cli` via `track:registry` into the **user-cache** area:

```
~/.cache/track/components/<tool.version>/track_cli.wasm
```

Project manifests pin the version:

```yaml
# track.yaml
tool:
  version: "0.1.0"
  # optional content digest for integrity checks
  digest: "<sha256>"
```

## Air-gapped options

### 1. Pre-warm user cache (recommended)

Copy a signed or checksum-verified `track_cli.wasm` onto the machine:

```bash
mkdir -p ~/.cache/track/components/0.1.0
cp track_cli.wasm ~/.cache/track/components/0.1.0/track_cli.wasm
```

Set `tool.digest` in `track.yaml` so the host rejects tampered artifacts.

### 2. Developer override

For local builds and CI, set:

```bash
export TRACK_CLI_COMPONENT=/path/to/track_cli.wasm
```

The host copies this file into the user cache on first resolve.

### 3. Project vendoring (future)

A later release may support vendoring under **project-cache**:

```
<project-root>/.track/components/<version>/track_cli.wasm
```

That path is not consulted in v0.1; use user-cache or `TRACK_CLI_COMPONENT` today.

## Host-only installs

Agent and CI images need only:

1. The `track` host binary
2. A writable user-cache directory (or pre-warmed component layer)
3. Optional pre-seeded `~/.config/track/config.json` for hub credentials

Each project directory pins its own `tool.version` without rebuilding images.
