//! A client mod starting, stopping and reading the recording of the world session's packets
//! (#19244): the platform's `/studio record`, asked for by a mod. The folder and the file name
//! are the platform's (`recordings_dir`, `generate_recording_filename`), the packets kept are
//! the recorder's privacy filter, and one recording runs at a time, whoever started it.
use super::Game;
use super::replay::StudioRecordError;
use horizon_client::file_id::valid_id;
use horizon_client::studio::{RecorderStatus, generate_recording_filename};
use mod_api::{RecordingError, RecordingStatus};
use std::path::Path;

/// The id the recording at `path` goes by, as `replay_open` takes it: its file name without
/// `.mcpr`; empty when no id can hold that name.
pub(super) fn recording_id_of(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".mcpr"))
        .filter(|id| valid_id(id))
        .map(str::to_owned)
        .unwrap_or_default()
}

/// What a mod reads of the recorder `status` says (none: no recording).
pub(super) fn status_of(status: Option<&RecorderStatus>) -> RecordingStatus {
    let Some(status) = status else {
        return mod_api::__private::built::recording_status(false, String::new(), 0, 0, false);
    };
    mod_api::__private::built::recording_status(
        status.active,
        recording_id_of(&status.path),
        u64::try_from(status.elapsed.as_millis()).unwrap_or(u64::MAX),
        status.bytes_written,
        status.error.is_some(),
    )
}

impl Game<'_> {
    /// How the session's recording stands, whoever started it.
    pub(super) fn recording_status(&self) -> RecordingStatus {
        let status = self
            .remote
            .as_ref()
            .and_then(|remote| remote.session.recorder_status());
        status_of(status.as_ref())
    }

    /// Starts recording the world session into the recordings folder, under a file name the
    /// platform makes from `name`; the recording's id.
    pub(super) fn recording_start(&mut self, name: Option<&str>) -> Result<String, RecordingError> {
        // A replay whose session has gone is no replay playing.
        self.drop_stale_replay();
        if self.replay.playing() {
            return Err(RecordingError::NotConnected);
        }
        let file_name = generate_recording_filename(name);
        let dir = self.recordings_dir();
        match self.begin_studio_recording(&dir, &file_name) {
            Ok(path) => Ok(recording_id_of(&path)),
            Err(StudioRecordError::NoSession) => Err(RecordingError::NotConnected),
            Err(StudioRecordError::Active) => Err(RecordingError::AlreadyRecording),
            Err(StudioRecordError::Disk(error)) => {
                log::warn!("A mod's recording could not be started: {error}");
                Err(RecordingError::DiskError)
            }
        }
    }

    /// Finishes the recording, whoever started it, and writes its archive; the recording's id.
    pub(super) fn recording_stop(&mut self) -> Result<String, RecordingError> {
        let Some(remote) = self.remote.as_ref() else {
            return Err(RecordingError::NotConnected);
        };
        match remote.session.stop_recording() {
            Some(Ok(path)) => Ok(recording_id_of(&path)),
            Some(Err(error)) => {
                log::warn!("A mod's recording ended with an error: {error}");
                Err(RecordingError::DiskError)
            }
            None => Err(RecordingError::NotRecording),
        }
    }
}
