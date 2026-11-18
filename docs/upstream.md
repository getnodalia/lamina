# Lámina upstream

| Field | Value |
|---|---|
| Upstream | `storytold/deckcraft` |
| Last observed | `0623b430324cf4c3d1962e6ae7885cf30c011c5e` |
| Last evaluated | `0623b430324cf4c3d1962e6ae7885cf30c011c5e` |
| Last integrated | `4aec38a2bceac18ee9c67b80b834441a69cc81ae` (209 commits since the last sync) on 2026-11-18 |
| Skipped | the community button and the brand files (Lámina has neither) |
| Adapted | every name (`scripts/rebrand.py`), the app icon, the Nodalia interface, PowerPoint as the format a new presentation is saved in |
| Conflicting | `README.md` (kept ours), `AGENTS.md` (kept ours) |

## How the sync works

Upstream's code arrives renamed: `scripts/rebrand.py` applies `.github/rebrand.toml` (DeckCraft
becomes Lámina, internal names included; links to the shared libraries point at
getnodalia/foundation) to upstream's tree, and the weekly sync
(`.github/workflows/upstream-sync.yml`) records the result as one mirror commit: its only parent is
the previous mirror commit, its subject is `Upstream <sha> with this app's names`, and an
`Upstream-Commit:` trailer names the upstream commit it renames. The sync merges that commit into a
branch and opens a pull request; files listed in `.github/upstream-keep` keep this repository's
version. No mirror branch is pushed, so merge sync pull requests with a merge commit: a squash would
drop the mirror commit the next sync starts from.

The mirror starts at the renamed `0623b430`, merged into `main` (with this repository's tree) by the
pull request that renamed the app. `python3 scripts/rebrand.py check` fails when an old name comes
back; a line that must keep one says `rebrand: keep`.

## Names kept on purpose

- The native file's signature (the manifest's `"format": "deckcraft"` and the zip's mimetype
  entry): every file saved so far carries it.
- `.deckcraft` and `.slidecraft` files still open; new ones are `.lamina`.
- Upstream's copyright line in `NOTICE` and the licences, which the licences require.
