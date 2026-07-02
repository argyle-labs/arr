<p align="center">
  <img src="assets/icon-256.png" width="120" alt="prowlarr" />
</p>

# prowlarr

Prowlarr is the **indexer manager** for the *arr apps — add your indexers/trackers
once here and it syncs them to Sonarr, Radarr, Lidarr, and Readarr.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend) — one cdylib in the [`arr`](../../README.md) workspace.

This repo is **self-contained** — the compose below runs prowlarr **by hand, without orca**.

---

## Run it without orca

### Docker Compose

Prowlarr manages indexers only — it needs no media or downloads mounts, just its `/config` volume.

```yaml
# compose.yml
services:
  prowlarr:
    image: lscr.io/linuxserver/prowlarr:latest
    container_name: prowlarr
    restart: unless-stopped
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    ports:
      - "9696:9696/tcp"    # web UI
    volumes:
      - ./config:/config    # database + settings (state)
```

```sh
docker compose up -d
```

Then add each app under *Settings → Apps* (Sonarr/Radarr/Lidarr/Readarr) so Prowlarr pushes indexers to them.

### Other runtimes

**Podman** — the compose above works with `podman compose up -d`.

**LXC / VM** — run the same image via Docker/Podman inside the guest, or install Prowlarr from upstream (<https://prowlarr.com/>).

**Unraid** — add from *Community Applications* (LinuxServer `prowlarr` template); point `/config` at `/mnt/user/appdata/prowlarr`.

### Dependencies

None to run. Optionally a [FlareSolverr](https://github.com/FlareSolverr/FlareSolverr) sidecar for Cloudflare-protected indexers. Prowlarr is itself the indexer dependency for [`sonarr`](../sonarr/README.md), [`radarr`](../radarr/README.md), [`lidarr`](../lidarr/README.md), and [`readarr`](../readarr/README.md).

### Ports & data

| | |
|---|---|
| Default port | `9696` |
| Image | `lscr.io/linuxserver/prowlarr` |
| State | the `/config` volume (SQLite db + settings) |
| Upstream | <https://prowlarr.com/> |
| Operator notes | [prowlarr.md](docs/prowlarr.md) |

### Backup & restore

- **Backup:** *System → Backup* in the UI writes a restorable `.zip`; back that up, or just copy the `/config` volume (stop the container first for a clean copy).
- **Restore:** put the `/config` volume back and start the container, or *System → Backup → Restore* a `.zip`.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where prowlarr runs.

## With orca

orca loads this cdylib as its own plugin exposing the `prowlarr.*` tools:

```sh
orca prowlarr.health <endpoint>          # health issues (typed payload)
orca prowlarr.indexers <endpoint>        # configured indexers
orca prowlarr.indexer_status <endpoint>  # per-indexer backoff status
orca prowlarr.test_indexers <endpoint>   # re-test indexers (clear stale backoff)
orca prowlarr.system_status <endpoint>   # app name / version / instance
```

## Layout

- `src/lib.rs` — the `prowlarr.*` tool surface (pure Rust) over the shared `arr_base` transport; carries the extra `indexer_status` tool.
- `src/abi_export.rs` — the cdylib FFI export (single `prowlarr.` namespace).
- `docs/` — standalone operator notes.
- `assets/` — plugin icon.
