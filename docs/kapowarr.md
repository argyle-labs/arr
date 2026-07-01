# Kapowarr

Comic and manga downloader. Monitors ComicVine for new issues and auto-downloads. Integrates with Komga.

- **Host**: <host> (<ip>)
- **Port**: 5656
- **Image**: `mrcas95/kapowarr`
- **Compose**: [compose/kapowarr/docker-compose.yml](../../compose/kapowarr/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/kapowarr` | `/config` | Kapowarr config and database |
| `/mnt/<host>/downloads` | `/downloads/completed` | Completed downloads (NFS) |
| `/mnt/<host>/data/media/comics` | `/data/media/comics` | Comics library (NFS) |
| `/mnt/<host>/data/media/manga` | `/data/media/manga` | Manga library (NFS) |

## Download Clients

Kapowarr has its own built-in downloader (uses getcomics.org and similar sources). Configure in Settings → Downloaders.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `KAPOWARR_IMAGE_TAG` | `latest` | Image tag |
| `KAPOWARR_CONFIG_PATH` | `/opt/appdata/kapowarr` | Config directory |
| `KAPOWARR_PORT` | `5656` | Host port |
| `DOWNLOADS_PATH` | `/mnt/<host>/downloads` | Downloads directory |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library base path |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs kapowarr
mount | grep <host>  # verify NFS mounts
```
