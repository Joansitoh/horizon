//! Host method declarations; access defaults and game forwarders are generated.
#![allow(unused_variables)]
use crate::*;
/// The client host's methods of the `recordings` area, implemented by the game and reached through [`crate::ClientHost`].
#[horizon_mod_api_macros::host_area]
pub trait RecordingsClientHost {
    /// Side: client.
    /// The recordings saved in the game folder's `recordings/`, newest first (by their `date`,
    /// then by id): what the recordings browser lists. Only a regular `.mcpr` file directly in
    /// the folder whose name is a recording id is listed, the files `replay_open` takes: a link,
    /// a file of another kind or a name no id can hold is left out, and so is a recording still
    /// being written, which has no file yet. The archives' metadata is read
    /// on the game's thread, so a folder of many recordings takes a moment. Empty while the
    /// folder is missing or empty.
    #[access(ClientSetup)]
    fn recordings(&self) -> Vec<RecordingInfo>;
    /// Side: client.
    /// Opens the recordings folder in the desktop's file manager, creating it first, as the
    /// recordings browser's button does. The folder opens beside the game: this does not wait
    /// for it. False when the folder could not be created or the desktop would not open it.
    #[access(WorldOnly)]
    fn open_recordings_folder(&mut self) -> bool;
}
