<p align="center">
  <img src="assets/icon-256.png" width="120" alt="arr" />
</p>

# arr

The *arr stack — Sonarr, Radarr, Prowlarr, Lidarr — automates finding and organizing TV, movies, and music.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend).

arr is multi-service — deploy it **by hand, without orca** from the upstream compose:

---

## Run it without orca

Follow the upstream install (which provides the official multi-container `docker-compose`): <https://wiki.servarr.com/>.


See [bazarr.md](docs/bazarr.md), [kapowarr.md](docs/kapowarr.md), [lidarr.md](docs/lidarr.md), [prowlarr.md](docs/prowlarr.md), [radarr-4k.md](docs/radarr-4k.md), [radarr.md](docs/radarr.md), [sonarr.md](docs/sonarr.md) for worked operator notes.


### Backup & restore

Back up the config/data volume(s) above — that's the whole service state (stop the container first for a clean copy). Restore by putting them back and starting it.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where arr runs. No per-service backup script.

## With orca

orca drives this plugin through the single generic `service.*` surface — no per-plugin tools:

```sh
orca service.deploy arr      # render + launch on any supported runtime
orca service.status arr      # health + rich diagnostics (typed payload)
orca service.backup arr      # location-agnostic backup (tar; PBS on Proxmox)
orca service.configure arr   # apply config via the upstream API
```

## Layout

- `src/` — the plugin (pure Rust): the `ServiceBackend` descriptor + `configure` / `status`.
- `docs/` — standalone operator notes.
- `assets/` — plugin icon.
