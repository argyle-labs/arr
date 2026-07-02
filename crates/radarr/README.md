<p align="center">
  <img src="assets/icon-256.png" width="120" alt="radarr" />
</p>

# radarr

Radarr automates finding, downloading, and organizing **movies**.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend) — one cdylib in the [`arr`](../../README.md) workspace.

This repo is **self-contained** — the compose below runs radarr **by hand, without orca**.

---

## Run it without orca

### Docker Compose

```yaml
# compose.yml
services:
  radarr:
    image: lscr.io/linuxserver/radarr:latest
    container_name: radarr
    restart: unless-stopped
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    ports:
      - "7878:7878/tcp"          # web UI
    volumes:
      - ./config:/config          # database + settings (state)
      - /mnt/media:/data/media    # media library (shared)
      - /mnt/downloads:/data/downloads  # completed downloads (shared)
```

```sh
docker compose up -d
```

Movie root folder: `/data/media/movies`. A separate **4K** instance is a common
second deployment — same image on port `7879` with its own `/config` and root
folder; see [radarr-4k.md](docs/radarr-4k.md).

### Other runtimes

**Podman** — the compose above works with `podman compose up -d`.

**LXC / VM** — run the same image via Docker/Podman inside the guest, or install Radarr from upstream (<https://radarr.video/>). Mount the shared media/downloads storage into the guest.

**Unraid** — add from *Community Applications* (LinuxServer `radarr` template); point `/config` at `/mnt/user/appdata/radarr` and the media/downloads mounts at your shares.

### Dependencies

- A download client — [`qbittorrent`](https://github.com/argyle-labs/qbittorrent) (torrent) and/or [`sabnzbd`](https://github.com/argyle-labs/sabnzbd) (usenet).
- Indexers — added once in [`prowlarr`](../prowlarr/README.md), which syncs them to Radarr.

### Ports & data

| | |
|---|---|
| Default port | `7878` (4K instance: `7879`) |
| Image | `lscr.io/linuxserver/radarr` |
| State | the `/config` volume (SQLite db + settings) |
| Upstream | <https://radarr.video/> |
| Operator notes | [radarr.md](docs/radarr.md) · [radarr-4k.md](docs/radarr-4k.md) |

### Backup & restore

- **Backup:** *System → Backup* in the UI writes a restorable `.zip`; back that up, or just copy the `/config` volume (stop the container first for a clean copy).
- **Restore:** put the `/config` volume back and start the container, or *System → Backup → Restore* a `.zip`.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where radarr runs.

## With orca

orca loads this cdylib as its own plugin exposing the `radarr.*` tools:

```sh
orca radarr.health <endpoint>          # health issues (typed payload)
orca radarr.indexers <endpoint>        # configured indexers
orca radarr.test_indexers <endpoint>   # re-test indexers (clear stale backoff)
orca radarr.system_status <endpoint>   # app name / version / instance
```

## Layout

- `src/lib.rs` — the `radarr.*` tool surface (pure Rust) over the shared `arr_base` transport.
- `src/abi_export.rs` — the cdylib FFI export (single `radarr.` namespace).
- `docs/` — standalone operator notes.
- `assets/` — plugin icon.
