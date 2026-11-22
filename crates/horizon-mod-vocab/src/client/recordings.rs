//! The saved recordings as a client mod lists them (`recordings`).

/// One saved recording in the recordings folder, as `recordings` lists it.
/// Mods receive this snapshot from the game; they do not construct it.
/// `#[non_exhaustive]` is for Rust callers only: on the wire every field is encoded and its
/// layout hashed, so adding one later breaks wasm mods built against this one
/// (docs/modding/src/concepts/wire-types.md).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
#[derive(horizon_mod_api_macros::Wire)]
pub struct RecordingInfo {
    /// The recording's id: its file name in the recordings folder without `.mcpr`, as
    /// `replay_open` takes it. Only recordings an id can name are listed.
    pub id: String,
    /// The name to show for it. Today the id, as the recordings browser titles it.
    pub name: String,
    /// The recorded length in ms, from the archive's metadata; 0 when that cannot be read.
    pub duration_ms: u32,
    /// The archive's size on disk, in bytes.
    pub size: u64,
    /// When the recording began, in ms since the Unix epoch, from the archive's metadata; the
    /// file's last change when that cannot be read.
    pub date: u64,
    /// Where it was recorded: the address of the server joined, `local` for a single-player
    /// world. Empty when the archive's metadata cannot be read.
    pub server: String,
    /// The Minecraft version it was recorded on, such as `26.3`. Empty when the archive's
    /// metadata cannot be read.
    pub version: String,
    /// It was recorded in a single-player world rather than on a server.
    pub singleplayer: bool,
}
