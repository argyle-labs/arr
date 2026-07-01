# Bazarr

Subtitle automation. Downloads subtitles for Sonarr and Radarr libraries.

- **Host**: <host> — see [Network Map](../network/network-map.md)
- **Port**: 6767
- **Image**: `lscr.io/linuxserver/bazarr`
- **Compose**: [compose/bazarr/docker-compose.yml](../../compose/bazarr/docker-compose.yml)
- **Network**: `media`

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/bazarr` | `/config` | Bazarr config |
| `/mnt/<host>/data/media` | `/data/media` | Media library (NFS) |

## Connected Apps

| App | Host | Port | Role |
|-----|------|------|------|
| Sonarr | <host> | 8989 | TV library source |
| Radarr | <host> | 7878 | Movie library source |
| Whisper ASR | <host> | 10300 | AI transcription fallback (NVIDIA GPU) |

## Subtitle Automation Policy

End state: **every video has an English subtitle**, every English subtitle is synced to the audio track, and the library carries no non-English sidecars.

### Providers

Configured in priority order (Bazarr falls back down the list):

1. **opensubtitles.com** — preferred. Real human transcripts, scored vs. release name.
2. **whisperai** — last-resort AI transcription. Runs on <host> RTX 3080 at `<ip>:10300` (container `whisper-ai`, image `onerahmet/openai-whisper-asr-webservice:latest-gpu`, model `medium / float16`, ~4 GB resident VRAM). Generates a sub when no provider hit clears `minimum_score=90`. Sample throughput: 15 s of audio → 1.5 s transcribe.

> **GPU sharing**: <host>'s 3080 is also used by `ollama` (`:11434`). With whisper-medium resident, ~5.8 GB free — enough for an 8B q4 LLM, NOT enough for `qwen3.5:9b` (6.6 GB). If you need that model, stop whisper-ai while ollama runs, or downsize whisper to `small`. See orca memory `project_<host>_gpu_contention.md`.

Whisper-generated subs are **not pinned**. `upgrade_subs=true` re-checks providers periodically; any later download that beats the current sub's score replaces it automatically.

### Scheduled tasks (Bazarr built-in)

| Task | Cadence | Purpose |
|------|---------|---------|
| `update_series` / `update_movies` | 60 min | Sync library from Sonarr/Radarr |
| `wanted_search_missing_subtitles_series` | 6 h | Hunt subs for episodes that have none (Whisper fills gaps) |
| `wanted_search_missing_subtitles_movies` | 6 h | Same, for movies |
| `series_full_scan_subtitles` / `movies_full_scan_subtitles` | daily 04:00 | Reindex on-disk sub files |
| `upgrade_subtitles` | 12 h | Re-score existing subs; replace if a better one is available. **Also re-runs ffsubsync** when the sub is out of sync threshold. `days_to_upgrade_subs=7`. |

### Sync behavior (ffsubsync)

- `use_subsync=true`, `force_audio=true`, `max_offset_seconds=60`
- `use_subsync_threshold=false` — sync **every** sub on upgrade pass, not just below-threshold ones
- Audio track is the reference (no transcript dependency)

### Sub-file slimming

Local convention enforced by [`scripts/bazarr-sub-cleanup.py`](../../scripts/bazarr-sub-cleanup.py):

**Keep**, per video:
- `<base>.en.<ext>` — primary English
- `<base>.en.forced.<ext>` — forced (foreign-dialog captions)
- `<base>.en.hi.<ext>` / `<base>.hi.<ext>` — English HI/SDH (standalone `.hi.srt` is treated as English-HI per spot-check, not Hindi)
- `<base>.<ext>` — unlabeled (assumed primary English)

**Delete**:
- Any sidecar in any other language (fr/es/de/zh/…)
- Duplicate English variants in the same slot (keeps largest)
- `*.bazarr.*` scratch files

Embedded subtitle tracks in the container are **never touched** — leaving baked-in foreign subs in place is fine.

Subtitle extensions handled: `.srt .ass .ssa .sub .idx .vtt .sup`

### Operational runbook

```bash
# Bazarr API key (also in container at /config/config/config.yaml)
K=$(ssh root@<ip> 'docker exec bazarr grep "^  apikey:" /config/config/config.yaml | head -1 | awk "{print \$2}"')
B=http://<ip>:6767/api

# Mass resync every existing English sub now (runs inside bazarr container,
# Bazarr throttles via concurrent_jobs=4)
scp scripts/bazarr-mass-resync.py root@<ip>:/root/
ssh root@<ip> 'docker cp /root/bazarr-mass-resync.py bazarr:/tmp/resync.py && \
  docker exec -e BAZARR_KEY='"$K"' bazarr python3 /tmp/resync.py'

# Slim non-English sidecars (RUN ON <host> — <host> NFS is flaky)
scp scripts/bazarr-sub-cleanup.py root@<host>:/root/
ssh root@<host> 'docker run --rm \
  -v /mnt/user/data/media:/data/media \
  -v /root/bazarr-sub-cleanup.py:/cleanup.py:ro \
  python:3.12-alpine python3 /cleanup.py --root /data/media'   # dry-run
# add --apply to actually delete

# Trigger Bazarr scheduled jobs ad-hoc
for jid in wanted_search_missing_subtitles_series \
          wanted_search_missing_subtitles_movies \
          upgrade_subtitles; do
  curl -s -X POST -H "X-API-KEY: $K" "$B/system/tasks?taskid=$jid"
done

# Update a Bazarr setting via API (form-encoded, lowercase bools)
curl -s -X POST -H "X-API-KEY: $K" \
  --data-urlencode 'settings-general-upgrade_subs=true' \
  "$B/system/settings"
```

### Future: orca arr-management plugin

Bazarr stays as the execution layer. Future orca *arr management plugins (bazarr/sonarr/radarr/prowlarr/lidarr) will expose this same policy surface — providers, schedules, bulk ops, sub-file hygiene — through orca's single pane of glass, so it's driven from CLI/MCP/REST/UI instead of the per-arr web UIs. See orca memory `project_orca_arr_management_plugins.md`.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `PUID` / `PGID` | `1000` / `100` | User/group ID |
| `BAZARR_IMAGE_TAG` | `latest` | Image tag |
| `BAZARR_CONFIG_PATH` | `/opt/appdata/bazarr` | Config directory |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs bazarr
# Config file (edit IPs here if API doesn't persist):
# /opt/appdata/bazarr/config/config.yaml
```
