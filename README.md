<p align="center">
  <img src="assets/icon-256.png" width="120" alt="arr" />
</p>

# arr

The *arr stack — **one repo, one orca plugin per app**. `arr` is a Cargo
workspace that builds an independent subprocess plugin for each *arr application;
orca spawns each on its own over the plugin socket. Every app sits on the shared [`arr-base`](crates/arr-base)
crate (the Servarr `X-Api-Key` transport + progenitor-generated typed clients),
so the ~590-line base stays DRY across all of them.

First-party [orca](https://github.com/argyle-labs/orca) plugins (service-backend).

## Plugins

| | App | Port | Role | Crate |
|---|-----|------|------|-------|
| <img src="crates/sonarr/assets/icon-256.png" width="28" /> | **Sonarr** | `8989` | TV series | [crates/sonarr](crates/sonarr) |
| <img src="crates/radarr/assets/icon-256.png" width="28" /> | **Radarr** | `7878` | Movies (4K: `7879`) | [crates/radarr](crates/radarr) |
| <img src="crates/prowlarr/assets/icon-256.png" width="28" /> | **Prowlarr** | `9696` | Source manager | [crates/prowlarr](crates/prowlarr) |
| <img src="crates/lidarr/assets/icon-256.png" width="28" /> | **Lidarr** | `8686` | Music | [crates/lidarr](crates/lidarr) |
| <img src="crates/readarr/assets/icon-256.png" width="28" /> | **Readarr** | `8787` | Books/audiobooks (legacy) | [crates/readarr](crates/readarr) |

Each crate is **self-contained**: its own README with standalone deploy /
backup / restore (run it by hand, without orca), its own operator notes, and its
own official icon.

## Run it without orca

Each app is an independent container. See the per-app README for its compose,
ports, and backup/restore — start with [`prowlarr`](crates/prowlarr) (add your
content sources there; it syncs them to the others), then the apps you want. All apps
share one media library and one media/imports folder and use the
[LinuxServer.io](https://docs.linuxserver.io/) images. Upstream: <https://wiki.servarr.com/>.

## Layout

```
arr/  (Cargo workspace)
  crates/
    arr-base/   shared Servarr transport + typed clients (rlib)
    sonarr/     -> sonarr    (subprocess plugin [[bin]])
    radarr/     -> radarr
    prowlarr/   -> prowlarr
    lidarr/     -> lidarr
    readarr/    -> readarr
```

### Adding an *arr app

1. Drop `specs/<app>.openapi.json` into [`arr-base/specs`](crates/arr-base/specs) and add a `flavor!(<app>)` (+ a `Flavor` variant if it uses the shared Servarr `Client`).
2. Add a `crates/<app>` subprocess-plugin crate (`[[bin]]` + `serve_tool_plugin!`) depending on `arr-base` with its own tool surface (single `<app>.` namespace).

Apps that aren't Servarr-shaped live in their own repos rather than this workspace, since they don't share the Servarr `Client` — e.g. [bazarr](https://github.com/argyle-labs/bazarr) and [kapowarr](https://github.com/argyle-labs/kapowarr).
