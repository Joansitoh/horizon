# Listing the saved recordings

A client mod lists the recordings saved in the game folder's `recordings/` and opens that folder, as the recordings
browser does. Native and wasm mods use the same calls; a wasm mod has no file access, so the platform reads the
folder and the archives' metadata for it.

| Call | Access | What it does |
|---|---|---|
| `recordings()` | client setup | The saved recordings, newest first, as `RecordingInfo` rows. Empty while the folder is missing or empty. |
| `open_recordings_folder()` | world | Creates the folder if needed and opens it in the desktop's file manager, without waiting for it. False when it could not be created or opened. |

`RecordingInfo` holds:

- `id`: the recording's file name without `.mcpr`, the id [`replay_open`](replay-playback.md) takes. `name` is what to
  show for it; today the same text.
- `duration_ms`, `date` (when it began, in ms since the Unix epoch), `server` (the address joined, `local` for a
  single-player world), `version` (the Minecraft version) and `singleplayer`, all from the archive's metadata.
- `size`: the archive's size on disk, in bytes.

Only what `replay_open` can open is listed: a regular `.mcpr` file directly in the folder whose name is a valid id.
A link, a folder, another kind of file and a name no id can hold are left out, and so is a recording still being
written, which has no file yet. An archive whose metadata cannot be read is still listed, with `duration_ms` 0,
`date` its file's last change, and an empty `server` and `version`. Rows come newest first by `date`, then by id.

The metadata is read from each archive on the game's thread when `recordings()` is called, so a folder of many
recordings takes a moment: call it when a browser opens or refreshes, not every frame.

`RecordingInfo` is `#[non_exhaustive]` for Rust callers only. On the wire every field is encoded and its layout
hashed ([wire types](../../concepts/wire-types.md)), so it stays as it is: adding a field would break wasm mods
built against this version.
