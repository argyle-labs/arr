<p align="center">
  <img src="assets/icon-256.png" width="120" alt="lidarr" />
</p>

# lidarr

Lidarr automates finding, organizing, and managing **music**.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend) — one cdylib in the [`arr`](../../README.md) workspace.

This repo is **self-contained** — the compose below runs lidarr **by hand, without orca**.

---

## Run it without orca

### Docker Compose

```yaml
# compose.yml
services:
  lidarr:
    image: lscr.io/linuxserver/lidarr:latest
    container_name: lidarr
    restart: unless-stopped
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    ports:
      - "8686:8686/tcp"          # web UI
    volumes:
      - ./config:/config          # database + settings (state)
      - /mnt/media:/data/media    # media library (shared)
      - /mnt/downloads:/data/downloads  # completed downloads (shared)
```

```sh
docker compose up -d
```

Music root folder: `/data/media/music`.

### Other runtimes

**Podman** — the compose above works with `podman compose up -d`.

**LXC / VM** — run the same image via Docker/Podman inside the guest, or install Lidarr from upstream (<https://lidarr.audio/>). Mount the shared media/downloads storage into the guest.

**Unraid** — add from *Community Applications* (LinuxServer `lidarr` template); point `/config` at `/mnt/user/appdata/lidarr` and the media/downloads mounts at your shares.

### Dependencies

- A client of your choice — your configured client.
- Content sources — added once in [`prowlarr`](../prowlarr/README.md), which syncs them to Lidarr.

### Ports & data

| | |
|---|---|
| Default port | `8686` |
| Image | `lscr.io/linuxserver/lidarr` |
| State | the `/config` volume (SQLite db + settings) |
| Upstream | <https://lidarr.audio/> |
| Operator notes | [lidarr.md](docs/lidarr.md) |

### Backup & restore

- **Backup:** *System → Backup* in the UI writes a restorable `.zip`; back that up, or just copy the `/config` volume (stop the container first for a clean copy).
- **Restore:** put the `/config` volume back and start the container, or *System → Backup → Restore* a `.zip`.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where lidarr runs.

## With orca

orca loads this cdylib as its own plugin exposing the `lidarr.*` tools:

```sh
orca lidarr.health <endpoint>          # health issues (typed payload)
orca lidarr.indexers <endpoint>        # configured indexers
orca lidarr.test_indexers <endpoint>   # re-test indexers (clear stale backoff)
orca lidarr.system_status <endpoint>   # app name / version / instance
```

## Layout

- `src/lib.rs` — the `lidarr.*` tool surface (pure Rust) over the shared `arr_base` transport.
- `src/abi_export.rs` — the cdylib FFI export (single `lidarr.` namespace).
- `docs/` — standalone operator notes.
- `assets/` — plugin icon.
