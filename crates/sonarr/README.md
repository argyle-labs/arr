<p align="center">
  <img src="assets/icon-256.png" width="120" alt="sonarr" />
</p>

# sonarr

Sonarr automates finding, downloading, and organizing **TV series**.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend) — one cdylib in the [`arr`](../../README.md) workspace.

This repo is **self-contained** — the compose below runs sonarr **by hand, without orca**.

---

## Run it without orca

### Docker Compose

```yaml
# compose.yml
services:
  sonarr:
    image: lscr.io/linuxserver/sonarr:latest
    container_name: sonarr
    restart: unless-stopped
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    ports:
      - "8989:8989/tcp"          # web UI
    volumes:
      - ./config:/config          # database + settings (state)
      - /mnt/media:/data/media    # media library (shared)
      - /mnt/downloads:/data/downloads  # completed downloads (shared)
```

```sh
docker compose up -d
```

TV root folder: `/data/media/tv`.

### Other runtimes

**Podman** — the compose above works with `podman compose up -d`.

**LXC / VM** — run the same image via Docker/Podman inside the guest, or install Sonarr from upstream (<https://sonarr.tv/>). Mount the shared media/downloads storage (e.g. an NFS/SMB share) into the guest.

**Unraid** — add from *Community Applications* (LinuxServer `sonarr` template); point `/config` at `/mnt/user/appdata/sonarr` and the media/downloads mounts at your shares.

### Dependencies

- A download client — [`qbittorrent`](https://github.com/argyle-labs/qbittorrent) (torrent) and/or [`sabnzbd`](https://github.com/argyle-labs/sabnzbd) (usenet).
- Indexers — added once in [`prowlarr`](../prowlarr/README.md), which syncs them to Sonarr.

### Ports & data

| | |
|---|---|
| Default port | `8989` |
| Image | `lscr.io/linuxserver/sonarr` |
| State | the `/config` volume (SQLite db + settings) |
| Upstream | <https://sonarr.tv/> |
| Operator notes | [sonarr.md](docs/sonarr.md) |

### Backup & restore

- **Backup:** *System → Backup* in the UI writes a restorable `.zip`; back that up, or just copy the `/config` volume (stop the container first for a clean copy).
- **Restore:** put the `/config` volume back and start the container, or *System → Backup → Restore* a `.zip`.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where sonarr runs.

## With orca

orca loads this cdylib as its own plugin exposing the `sonarr.*` tools:

```sh
orca sonarr.health <endpoint>          # health issues (typed payload)
orca sonarr.indexers <endpoint>        # configured indexers
orca sonarr.test_indexers <endpoint>   # re-test indexers (clear stale backoff)
orca sonarr.system_status <endpoint>   # app name / version / instance
```

## Layout

- `src/lib.rs` — the `sonarr.*` tool surface (pure Rust) over the shared `arr_base` transport.
- `src/abi_export.rs` — the cdylib FFI export (single `sonarr.` namespace).
- `docs/` — standalone operator notes.
- `assets/` — plugin icon.
