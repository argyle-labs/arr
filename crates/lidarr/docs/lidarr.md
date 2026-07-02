# Lidarr

Music automation. Monitors, downloads, and organizes music.

- **Host**: <host> (<ip>)
- **Port**: 8686
- **Image**: `lscr.io/linuxserver/lidarr`
- **Compose**: [compose/lidarr/docker-compose.yml](../../compose/lidarr/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/lidarr` | `/config` | Lidarr config |
| `/mnt/<host>/downloads` | `/downloads/completed` | Completed downloads (NFS) |
| `/mnt/<host>/data/media` | `/data/media` | Media library (NFS) |

## Media Root Folder

`/data/media/music`

## Download Clients

| Client | Protocol | Host | Port |
|--------|----------|------|------|
| SABnzbd | NZB | <ip> | 8080 |
| qBittorrent | Torrent | <ip> | 8070 |

## Indexers

Managed by Prowlarr (fullSync). See [prowlarr.md](prowlarr.md).

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `PUID` / `PGID` | `1000` / `100` | User/group ID (100 = Unraid users group) |
| `LIDARR_IMAGE_TAG` | `latest` | Image tag |
| `LIDARR_CONFIG_PATH` | `/opt/appdata/lidarr` | Config directory |
| `DOWNLOADS_PATH` | `/mnt/<host>/downloads` | Downloads directory |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs lidarr
mount | grep <host>  # verify NFS mounts
```
