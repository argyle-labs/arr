# Readarr

Book & audiobook automation. Monitors, downloads, and organizes ebooks and audiobooks.

- **Port**: 8787
- **Image**: `lscr.io/linuxserver/readarr` (`develop` tag; see note below)
- **API**: `/api/v1`, `X-Api-Key` auth (Servarr family)
- **Network**: `media`

## Status: legacy

Upstream Readarr development was **discontinued** by the Servarr team — the
project is archived and no longer actively maintained. The LinuxServer image
remains available on the `develop` tag and existing installs keep working, but
expect no new upstream releases. This plugin still speaks the standard Servarr
`/api/v1` surface (health / indexers / system status), so it manages a running
Readarr instance exactly like the other flavors.

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `./config` | `/config` | Readarr config + database |
| `/mnt/downloads` | `/data/downloads` | Completed downloads (shared) |
| `/mnt/media` | `/data/media` | Media library (shared) |

## Root Folders

- Ebooks: `/data/media/books`
- Audiobooks: `/data/media/audiobooks`

## Download Clients

A torrent (`qbittorrent`) and/or usenet (`sabnzbd`) client, same as the other flavors.

## Indexers

Managed by Prowlarr (add Readarr under *Prowlarr → Settings → Apps*). See [prowlarr.md](../../prowlarr/docs/prowlarr.md).

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `PUID` / `PGID` | `1000` / `1000` | User/group ID |
