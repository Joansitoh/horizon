//! Host method declarations; access defaults and game forwarders are generated.
#![allow(unused_variables)]
use crate::*;
/// The client host's methods of the `recording` area, implemented by the game and reached through [`crate::ClientHost`].
#[horizon_mod_api_macros::host_area]
pub trait RecordingClientHost {
    /// Side: client.
    /// How the session's recording stands now, whoever started it (a mod or `/studio record`):
    /// all fields empty or false while no recording runs.
    #[access(ClientSetup)]
    fn recording_status(&self) -> RecordingStatus;
    /// Side: client.
    /// Starts recording the world session's packets, as `/studio record start` does: the
    /// platform picks the folder (the game folder's `recordings/`) and the file name, which
    /// begins with `name` when one is given (its letters, digits, `-` and `_` kept, whitespace
    /// made `_`, at most 80 characters) and ends with the time and a counter, so it is never a
    /// path and never names another recording. What the recording keeps is the platform's
    /// privacy filter, unchanged. Answers the recording's id, the one `recording_stop` gives and
    /// `replay_open` takes once the recording has ended.
    ///
    /// One recording runs at a time: `AlreadyRecording` while one does, a recorder that has
    /// failed (`RecordingStatus::failed`) excepted, which a new recording replaces. `NotConnected`
    /// with no world session, or while a replay plays. `DiskError` when the file cannot be
    /// created.
    #[access(WorldOnly)]
    fn recording_start(&mut self, name: Option<&str>) -> Result<String, RecordingError>;
    /// Side: client.
    /// Stops the recording and finishes its `.mcpr` archive, whoever started it: the file is in
    /// the recordings folder when this answers. Answers the recording's id, as `replay_open` takes
    /// it (empty for a recording started by `/studio record start` under a name no id can hold).
    /// The archive is written on the game's thread before this answers, so a long recording takes a moment.
    /// `NotRecording` while no recording runs, `NotConnected` with no world session or, when no
    /// recording runs, while a replay plays (as `recording_start` answers during a replay),
    /// `DiskError` when the recording had failed or its archive could not be finished.
    #[access(WorldOnly)]
    fn recording_stop(&mut self) -> Result<String, RecordingError>;
}
