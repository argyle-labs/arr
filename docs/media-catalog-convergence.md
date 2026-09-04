# Unified media catalog + convergence (mapping / long-term)

> **Status: MAP ONLY.** The unifying orca `media` catalog domain is **not built
> yet**. This is the spec. It is the *catalog* half of the unified media domain;
> the *playback* half ("any source → any player") is in
> [homeassistant/docs/music-assistant.md](../../homeassistant/docs/music-assistant.md).

## Goal

**One holistic, de-duplicated view of our media** — every movie / 4K movie / TV
show / episode as a single canonical **title**, no matter how many places it
physically lives. From that one view: add new titles, manage subtitles, see where
each title exists (and where it's *missing*), and play it on any system.

Today the same title is scattered across, and duplicated between:

- **Files** on the willow share (`/data/media/{movies,4k,tv}`)
- **Managers**: Sonarr (TV), Radarr (movies), Radarr-4K (4K movies) — acquisition + file-of-record
- **Servers**: Plex **mimir**, Plex **njord**, **Jellyfin** — serving/playback
- **Bazarr**: subtitles

A movie can be a Radarr entry + a file + a mimir item + a njord item + a Jellyfin
item + N subtitle tracks — that's **one title, six+ references**, not six things.

## De-dupe model = canonical identity + reference merge

This is orca's existing identity model applied to media
([[canonical-identity-and-reference-merge]], [[locality-is-a-flag-not-an-identity]],
[[reference-fields-are-nested-id-not-thingId]]):

- **Canonical title** keyed by **external IDs** — `imdb` / `tmdb` (movies),
  `tvdb` / `imdb` (shows+episodes). Every surface exposes these (arrs store them;
  Plex/JF GUIDs embed them), so the join is deterministic — that's the de-dupe key.
- **References** hang off the canonical title, each a locality-flagged pointer:

| Reference | Source | Carries |
|-----------|--------|---------|
| manager entry | sonarr / radarr / radarr-4k | monitored?, quality profile, wanted/missing, file path |
| file(s) | willow share | path, size, resolution, codec, owner (see [[smb-ownership-login-user-not-mount-uid]]) |
| plex item ×2 | mimir, njord | ratingKey, library, watched state, available? |
| jellyfin item | jellyfin | item id, library, available? |
| subtitles | bazarr | languages, forced/hi, source, present/missing |

Merging by external ID collapses the duplicates into one row with a reference set
— the "wholistic view."

## Capabilities (what the unified view enables)

- **Add a title** → routed to the right manager by type: TV→Sonarr, movie→Radarr,
  4K movie→Radarr-4K. (One `media.title.add {type}`; orca picks the backend.)
- **Manage subtitles** → Bazarr: see per-title subtitle state, request/upgrade,
  fix drift. See [subtitle-integrity-plugins.md](subtitle-integrity-plugins.md).
- **Play it** → hand the canonical title to the playback domain
  ([media-playback](../../homeassistant/docs/music-assistant.md)): any source
  (mimir/njord/JF) → any player.
- **Converge / detect drift** across the reference set — the high-value part:
  - title in mimir but **not njord** (or not Jellyfin) → servers out of sync
  - manager says imported but **no server item** → scan/notify gap (ties to
    [[media-scan-on-import-dispatcher]] + [[plex-reclaim-breaks-arr-notifications]])
  - **file on disk with no manager entry** → unmanaged/orphan
  - manager entry **wanted/missing** → acquisition gap
  - **missing/only-forced subtitles** → Bazarr gap
  - same title at **different quality/resolution** across servers → which is canonical
  - **watched/progress state out of sync** across mimir / njord / Jellyfin → converge

### Watched / progress-state sync

Watched status and resume position are per-title reference attributes that must
stay consistent across **all serving surfaces** — Plex **mimir**, Plex **njord**,
and **Jellyfin** — so "where you left off" follows you regardless of which server
a device happens to use. This is another **convergence loop**: read watched/position
per (title × server), pick the freshest, write it back to the laggards (guard
against clobbering a newer in-progress state).

Today this is a standalone **`jellyplex-watched`** container (Plex↔Jellyfin↔Emby,
its own tokens/env). orca should **own it as a first-class capability**, not a
black box: watched-state is a title attribute in this catalog, and the sync is a
`media.title.converge` concern (creds from the orca vault, drift surfaced in the
unified view). Fold the `jellyplex-watched` plugin's job into the `media` domain.

## Proposed orca surface

Extends the `media.*` domain (catalog side):
`media.title.list [--type movie|4k|show] [--missing] [--drift]`,
`media.title.detail {id}` (full reference set + subtitle state + availability per
server), `media.title.add {type, extId}`, `media.title.subtitles {id, …}`,
`media.title.converge {id|all}` (reconcile servers/subs/files), and it feeds
`media.play` (playback half). Read path is a live join across the backends
(on-demand, not a poll-cache — [[on-demand-not-poll-and-cache]]).

## orca plugin plan

- **arr plugins** (this repo, one per app on `arr-base`) expose each manager's
  titles + external IDs + add/monitor — the acquisition + file-of-record side.
- **plex / jellyfin plugins** expose their library items + GUIDs + availability +
  watched state.
- **bazarr plugin** exposes per-title subtitle state + request/upgrade.
- **orca core** owns the **canonical merge** (join by external ID → one title,
  many references) and the **convergence/drift** logic — generic identity model,
  not per-plugin ([[orca-defines-what-plugins-define-how]],
  [[generic-core-tables-not-per-plugin]]). Same single-pane vision as everything
  else ([[orca-single-pane-of-glass-vision]]).

## Build phases (incremental / additive)

1. **Read + merge**: each backend lists titles with external IDs; orca joins → the
   de-duped catalog view (`media.title.list/detail`).
2. **Drift detection**: surface the gaps above (missing on a server, orphan files,
   missing subs, wanted).
3. **Actions**: `add` (route by type), `subtitles` (Bazarr), `converge` (reconcile
   servers/subs, reusing the scan dispatcher).
4. **Join to playback**: `media.play` from a canonical title.
5. **Codify** as the orca `media` domain + per-backend plugins.
