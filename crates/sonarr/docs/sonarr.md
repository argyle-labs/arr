# Sonarr

TV show automation. Monitors, downloads, and organizes TV shows.

- **Host**: <host> — see [Network Map](../network/network-map.md)
- **Port**: 8989
- **Image**: `lscr.io/linuxserver/sonarr`
- **Compose**: [compose/sonarr/docker-compose.yml](../../compose/sonarr/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/sonarr` | `/config` | Sonarr config |
| `/mnt/<host>/downloads` | `/downloads/completed` | Completed downloads (NFS) |
| `/mnt/<host>/data/media` | `/data/media` | Media library (NFS) |

## Media Root Folder

`/data/media/tv`

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
| `SONARR_IMAGE_TAG` | `latest` | Image tag |
| `SONARR_CONFIG_PATH` | `/opt/appdata/sonarr` | Config directory |
| `DOWNLOADS_PATH` | `/mnt/<host>/downloads` | Downloads directory |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs sonarr
mount | grep <host>  # verify NFS mounts
```
