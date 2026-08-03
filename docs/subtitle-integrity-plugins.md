# Subtitle & File Integrity — Sonarr + Bazarr plugin outline

Design outline for the **detect → diagnose → remediate** capabilities the
Sonarr and Bazarr orca plugins must expose so problems like the one below are
caught and fixed automatically, not by hand.

## Motivating incident (real)

*The Venture Bros. — S02E01 "Powerless in the Face of Death".* Reported as
"wrong file or wrong subtitle." Investigation found:

- **Video was correct.** Its embedded French track opened at `00:00:12` with
  the Impossibles / costume-contest flashback that opens the episode; filename,
  `.nfo`, and Sonarr all agreed.
- **External `.en.srt` was wrong** — first cue at `00:00:36`, ended `61 s`
  before the video's `22:43` runtime (a ~24 s desync onto a different cut). A
  mismatched sidecar, not a mislabeled file.
- Series-wide sweep then surfaced the larger latent problem: **57 episodes with
  no English subtitle at all** (S3–S5, S7, specials), and only **41** of them
  were even in Bazarr's "wanted" queue — ~16 were ineligible due to a language
  profile / monitoring gap Bazarr will never self-heal.

Every one of these signals is machine-checkable. That is the plugins' job.

## Division of responsibility

| Concern | Owner | Why |
|---------|-------|-----|
| Is the **video** the episode it claims? | **Sonarr** | Sonarr owns the file, its release name, and re-grab. |
| Is the **subtitle** present / correct / synced? | **Bazarr** | Bazarr owns providers, scoring, ffsubsync. |
| Language-profile / monitoring coverage gaps | **both** | Sonarr monitors the episode; Bazarr assigns the language profile. |

Both surface through the standard orca **diagnostics** (`diagnose`) and
**repair** contracts plus the six-verb data surface — no bespoke tools.

---

## Bazarr plugin

Bazarr has **no Servarr-shaped API** (own REST + `X-API-KEY`, different
resource model), so it does **not** sit on `arr-base`'s progenitor client — it
gets its own thin client crate. See [bazarr.md](bazarr.md) for the live policy.

### Detect (six-verb `List` / `Detail`)

- `List` episodes/movies with subtitle state per language (present, provider,
  score, path).
- `Detail` on an episode returns the integrity facts below.

### Diagnose (`diagnostics::diagnose` → `Issue`s)

Each check is language-independent unless noted and maps to one `Issue`:

| Issue | Check | Threshold |
|-------|-------|-----------|
| `SubtitleMissing` | no sidecar for a profile language | — |
| `SubtitleDesynced` | `srt_last_cue` vs container `duration` | end **after** video, or > ~90 s early |
| `SubtitleWrongContent` | first-cue content/timing vs embedded reference track (or audio via Whisper spot-check) | first cue offset ≫ reference |
| `SubtitleNotWanted` | episode missing a sub but **absent from `wanted`** | coverage gap |
| `LanguageProfileGap` | monitored episode with no language profile assigned | — |
| `ForeignSidecarPresent` | non-profile-language sidecar left on disk | policy: English-only |

The `srt_end vs duration` and `first-cue vs embedded-track` heuristics are
exactly what caught S02E01 — implement them as the core of `SubtitleDesynced` /
`SubtitleWrongContent`.

### Remediate (`diagnostics::repair` / six-verb `Delete`+`Upsert`)

- `SubtitleMissing` / `SubtitleNotWanted` → trigger provider search
  (`PATCH episodes/subtitles`) or the `wanted_search_missing_subtitles_series`
  task; fall back to Whisper (`whisperai`) when no provider clears
  `minimum_score`.
- `SubtitleDesynced` → run ffsubsync (`subsync.*`, audio reference) rather than
  re-fetch when content is right (e.g. S02E07 "Love-Bheits" — correct sub,
  shorter video cut).
- `SubtitleWrongContent` → blacklist + re-fetch best-scored replacement.
- `ForeignSidecarPresent` → `Delete`.
- `LanguageProfileGap` → assign the default profile (repair may require
  confirmation — it changes Sonarr-side config).

> **Idempotent + confirm-before-mutate.** Repairs touch the shared media
> library over NFS. Destructive steps (delete, blacklist, profile change)
> confirm unless run under an explicit `--apply`/autoheal policy.

---

## Sonarr plugin

Sonarr already ships as a crate ([`crates/sonarr`](../crates/sonarr)). Extend it
with the file-integrity half:

### Diagnose

| Issue | Check |
|-------|-------|
| `EpisodeFileMissing` | monitored, aired, no file |
| `EpisodeFileMismatch` | on-disk media doesn't match the episode it's mapped to — runtime vs. TVDB runtime, embedded title/track content, or hash of a known-bad grab |
| `QualityCutMismatch` | file runtime materially shorter/longer than siblings (the S02E07 short-cut signal) |
| `UnparsedRelease` | file Sonarr couldn't map to an episode |

### Remediate

- `EpisodeFileMissing` / `EpisodeFileMismatch` → trigger **search + grab**
  (`EpisodeSearch` command) to pull a correct release; optionally
  `DELETE episodefile` on the bad one first.
- `QualityCutMismatch` → flag for manual review or re-grab at a preferred
  quality profile.

### "Pull the file" path the user asked for

`sonarr` exposes the re-grab as a repair action: given a `seriesId`+`episodeId`,
issue `EpisodeSearch`, watch the queue, and report the grabbed release. This is
the remediation when the **video** (not the subtitle) is the wrong file.

---

## Cross-plugin flow (autoheal)

```
Sonarr.diagnose  ─┐
                  ├─► orca correlates by (series, episode)
Bazarr.diagnose  ─┘
        │
        ├─ video wrong  → Sonarr.repair (re-grab) → Bazarr.repair (re-fetch sub)
        ├─ sub wrong    → Bazarr.repair (blacklist + re-fetch)
        ├─ sub desynced → Bazarr.repair (ffsubsync)
        └─ coverage gap → Sonarr/Bazarr.repair (monitor + language profile)
```

A scheduled orca job runs both `diagnose` passes across the library and emits a
single integrity report; `repair` runs under policy (dry-run by default).

## Build order

1. Bazarr client crate + `List`/`Detail` subtitle state (no `arr-base`).
2. Bazarr `diagnose` (the six checks above) — highest leverage, read-only.
3. Bazarr `repair` (search / sync / blacklist / delete).
4. Sonarr file-integrity `diagnose` + re-grab `repair`.
5. orca cross-plugin correlation + scheduled autoheal.
