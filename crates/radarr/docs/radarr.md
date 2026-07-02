# Radarr

Movie automation. Monitors, downloads, and organizes movies.

- **Host**: <host> — see [Network Map](../network/network-map.md)
- **Port**: 7878
- **Image**: `lscr.io/linuxserver/radarr`
- **Compose**: [compose/radarr/docker-compose.yml](../../compose/radarr/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/radarr` | `/config` | Radarr config |
| `/mnt/<host>/downloads` | `/downloads/completed` | Completed downloads (NFS) |
| `/mnt/<host>/data/media` | `/data/media` | Media library (NFS) |

## Media Root Folder

`/data/media/movies`

## Download Clients

| Client | Protocol | Host | Port |
|--------|----------|------|------|
| SABnzbd | NZB | <host> | 8080 |
| qBittorrent | Torrent | <host> | 8070 |

## Indexers

Managed by Prowlarr (fullSync). See [prowlarr.md](prowlarr.md).

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `PUID` / `PGID` | `1000` / `100` | User/group ID (100 = Unraid users group) |
| `RADARR_IMAGE_TAG` | `latest` | Image tag |
| `RADARR_CONFIG_PATH` | `/opt/appdata/radarr` | Config directory |
| `DOWNLOADS_PATH` | `/mnt/<host>/downloads` | Downloads directory |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs radarr
mount | grep <host>  # verify NFS mounts
```
