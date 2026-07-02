<p align="center">
  <img src="assets/icon-256.png" width="120" alt="readarr" />
</p>

# readarr

Readarr automates finding, downloading, and organizing **ebooks and audiobooks**.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend) — one cdylib in the [`arr`](../../README.md) workspace.

> **Legacy:** upstream Readarr development has been discontinued (the Servarr project is archived). The LinuxServer image and existing installs keep working; this plugin speaks the standard Servarr `/api/v1` surface. See [readarr.md](docs/readarr.md).

This repo is **self-contained** — the compose below runs readarr **by hand, without orca**.

---

## Run it without orca

### Docker Compose

```yaml
# compose.yml
services:
  readarr:
    image: lscr.io/linuxserver/readarr:develop
    container_name: readarr
    restart: unless-stopped
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    ports:
      - "8787:8787/tcp"          # web UI
    volumes:
      - ./config:/config          # database + settings (state)
      - /mnt/media:/data/media    # media library (shared)
      - /mnt/downloads:/data/downloads  # completed downloads (shared)
```

```sh
docker compose up -d
```

Root folders: `/data/media/books` (ebooks) and `/data/media/audiobooks`.

### Other runtimes

**Podman** — the compose above works with `podman compose up -d`.

**LXC / VM** — run the same image via Docker/Podman inside the guest. Mount the shared media/downloads storage into the guest.

**Unraid** — add from *Community Applications* (LinuxServer `readarr` template); point `/config` at `/mnt/user/appdata/readarr` and the media/downloads mounts at your shares.

### Dependencies

- A download client — [`qbittorrent`](https://github.com/argyle-labs/qbittorrent) (torrent) and/or [`sabnzbd`](https://github.com/argyle-labs/sabnzbd) (usenet).
- Indexers — added once in [`prowlarr`](../prowlarr/README.md), which syncs them to Readarr.

### Ports & data

| | |
|---|---|
| Default port | `8787` |
| Image | `lscr.io/linuxserver/readarr` (`develop`) |
| State | the `/config` volume (SQLite db + settings) |
| Upstream | archived — <https://github.com/Readarr/Readarr> |
| Operator notes | [readarr.md](docs/readarr.md) |

### Backup & restore

- **Backup:** *System → Backup* in the UI writes a restorable `.zip`; back that up, or just copy the `/config` volume (stop the container first for a clean copy).
- **Restore:** put the `/config` volume back and start the container, or *System → Backup → Restore* a `.zip`.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where readarr runs.

## With orca

orca loads this cdylib as its own plugin exposing the `readarr.*` tools:

```sh
orca readarr.health <endpoint>          # health issues (typed payload)
orca readarr.indexers <endpoint>        # configured indexers
orca readarr.test_indexers <endpoint>   # re-test indexers (clear stale backoff)
orca readarr.system_status <endpoint>   # app name / version / instance
```

## Layout

- `src/lib.rs` — the `readarr.*` tool surface (pure Rust) over the shared `arr_base` transport.
- `src/abi_export.rs` — the cdylib FFI export (single `readarr.` namespace).
- `docs/` — standalone operator notes.
- `assets/` — plugin icon.
