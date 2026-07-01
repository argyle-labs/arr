# Prowlarr

Indexer manager. Syncs indexers to Sonarr, Radarr, and Radarr 4K.

- **Host**: <host> — see [Network Map](../network/network-map.md)
- **Port**: 9696
- **Image**: `lscr.io/linuxserver/prowlarr`
- **Compose**: [compose/prowlarr/docker-compose.yml](../../compose/prowlarr/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/prowlarr` | `/config` | Prowlarr config |

## Connected Apps

| App | URL | Sync |
|-----|-----|------|
| Sonarr | http://<host>:8989 | fullSync |
| Radarr | http://<host>:7878 | fullSync |
| Radarr 4K | http://<host>:7879 | fullSync |
| Lidarr | http://<host>:8686 | fullSync |

> LazyLibrarian does **not** sync from Prowlarr — configure indexers directly in its settings using Prowlarr's Newznab/Torznab proxy URLs.

Prowlarr URL (used by apps to pull indexers): `http://<host>:9696`

## Indexers

NZBgeek is the primary source (priority 1). All public trackers are priority 25.

| Indexer | Type | Priority | Notes |
|---------|------|----------|-------|
| NZBgeek | Newznab (NZB) | 1 | Primary — best retention, all content types |
| The Pirate Bay | Torrent | 25 | General |
| YTS | Torrent | 25 | Movies only — high quality encodes |
| LimeTorrents | Torrent | 25 | General — TV, movies |
| Comicat | Torrent | 25 | Comics |

### Not yet working (require FlareSolverr)

| Indexer | Blocker |
|---------|---------|
| EZTV / EZTVL | Cloudflare — needs FlareSolverr container |
| 1337x | Cloudflare — needs FlareSolverr container |

### Pending — private tracker accounts needed

| Indexer | Content |
|---------|---------|
| MyAnonamouse (MAM) | Ebooks + audiobooks (best source) |
| Redacted (RED) / Orpheus (OPS) | Music |
| Nyaa.si | Manga/anime — currently timing out through VPN |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `PUID` / `PGID` | `1000` / `100` | User/group ID |
| `PROWLARR_IMAGE_TAG` | `latest` | Image tag |
| `PROWLARR_CONFIG_PATH` | `/opt/appdata/prowlarr` | Config directory |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs prowlarr
# Force sync indexers to all apps:
curl -X POST http://<host>:9696/api/v1/applications/sync -H "X-Api-Key: <key>"
```
