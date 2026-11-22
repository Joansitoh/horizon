//! A client mod listing the saved recordings and opening their folder (#19245): the data the
//! recordings browser (`studio_ui.rs`) reads, by the ids `replay_open` takes.
use super::Game;
use super::native_replay::recording_path;
use horizon_client_render::studio_ui::StudioUi;
use mod_api::RecordingInfo;
use std::path::Path;

/// The recordings in `dir`, newest first (by date, then id): every regular `.mcpr` file directly
/// in it whose name is a recording id, which `recording_path` decides as `replay_open` does (a
/// link, another kind of file, a name no id can hold, or a `dir` that is itself a link, lists
/// nothing). An archive whose metadata can't be read is listed with what the file says: its size
/// and last change.
pub(super) fn list_recordings(dir: &Path) -> Vec<RecordingInfo> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(id) = file_name
            .to_str()
            .and_then(|name| name.strip_suffix(".mcpr"))
        else {
            continue;
        };
        let Ok(path) = recording_path(dir, id) else {
            continue;
        };
        let Ok(file) = std::fs::metadata(&path) else {
            continue;
        };
        let (duration_ms, date, server, version, singleplayer) =
            match StudioUi::read_metadata(&path) {
                Some(meta) => (
                    meta.duration,
                    meta.date,
                    meta.server_name,
                    meta.mcversion,
                    meta.singleplayer,
                ),
                None => {
                    let changed = file
                        .modified()
                        .ok()
                        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                        .map_or(0, |since| {
                            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
                        });
                    (0, changed, String::new(), String::new(), false)
                }
            };
        found.push(mod_api::__private::built::recording_info(
            id.to_owned(),
            id.to_owned(),
            duration_ms,
            [file.len(), date],
            [server, version],
            singleplayer,
        ));
    }
    found.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id)));
    found
}

impl Game<'_> {
    /// The saved recordings, for a client mod.
    pub(super) fn recordings(&self) -> Vec<RecordingInfo> {
        list_recordings(&self.recordings_dir())
    }

    /// Opens the recordings folder in the file manager, creating it first, as the browser's
    /// button does; whether the desktop took it.
    pub(super) fn open_recordings_folder(&mut self) -> bool {
        let dir = self.recordings_dir();
        if let Err(error) = std::fs::create_dir_all(&dir) {
            log::warn!("A mod could not create the recordings folder: {error}");
            return false;
        }
        match horizon_client::desktop::open_path(&dir) {
            Ok(()) => true,
            Err(error) => {
                log::warn!("A mod could not open {}: {error}", dir.display());
                false
            }
        }
    }
}
