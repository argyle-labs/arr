# Radarr 4K

Separate Radarr instance for 4K content.

- **Host**: <host> — see [Network Map](../network/network-map.md)
- **Port**: 7879 (external) → 7878 (internal)
- **Image**: `lscr.io/linuxserver/radarr`
- **Compose**: [compose/radarr-4k/docker-compose.yml](../../compose/radarr-4k/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/radarr-4k` | `/config` | Radarr 4K config |
| `/mnt/<host>/downloads` | `/downloads/completed` | Completed downloads (NFS) |
| `/mnt/<host>/data/media` | `/data/media` | Media library (NFS) |

## Media Root Folder

`/data/media/4k`

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
| `RADARR_4K_IMAGE_TAG` | `latest` | Image tag (falls back to `RADARR_IMAGE_TAG`) |
| `RADARR_4K_CONFIG_PATH` | `/opt/appdata/radarr-4k` | Config directory |
| `DOWNLOADS_PATH` | `/mnt/<host>/downloads` | Downloads directory |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

> Port note: Radarr listens on 7878 internally. Compose maps external 7879 → internal 7878.

## Troubleshooting

```bash
docker logs radarr-4k
mount | grep <host>  # verify NFS mounts
```
