//! A client mod starts, stops and reads the recording of the world session (#19244).
use super::native_recording::{recording_id_of, status_of};
use super::native_replay::recording_path;
use horizon_client::file_id::valid_id;
use horizon_client::studio::{RecorderStatus, generate_recording_filename};
use mod_api::RecordingError;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The platform makes the file name from the mod's `name`: only letters, digits, `-` and `_`
/// survive (whitespace becomes `_`, at most 80 characters), so a name is never a path, and the
/// file name's stem is always an id `replay_open` takes.
#[test]
fn a_recording_name_never_escapes_the_recordings_folder() {
    let long = "a".repeat(200);
    let cases = [
        (Some("my trip!"), "my_trip_"),
        (Some("../../etc/passwd"), "etcpasswd_"),
        (Some("C:\\temp\\x"), "Ctempx_"),
        (Some("CON"), "CON_"),
        (Some("!!!"), "recording_"),
        (Some("  "), "recording_"),
        (Some(""), "recording_"),
        (None, "recording_"),
    ];
    for (name, prefix) in cases {
        let file = generate_recording_filename(name);
        let id = file.strip_suffix(".mcpr").expect("an .mcpr file");
        assert!(id.starts_with(prefix), "{name:?} made {file}");
        assert!(valid_id(id), "{name:?} made {file}");
        assert_eq!(
            recording_id_of(&Path::new("recordings").join(&file)),
            id,
            "{name:?}"
        );
    }
    let file = generate_recording_filename(Some(long.as_str()));
    assert!(
        file.starts_with(&format!("{}_", "a".repeat(80))),
        "at most 80 characters of the name: {file}"
    );
}

/// Two recordings started in the same millisecond still get two files.
#[test]
fn two_recordings_of_one_name_get_two_files() {
    let first = generate_recording_filename(Some("trip"));
    let second = generate_recording_filename(Some("trip"));
    assert_ne!(first, second);
}

fn recorder_status(active: bool, path: &str, error: Option<&str>) -> RecorderStatus {
    RecorderStatus {
        active,
        path: PathBuf::from(path),
        elapsed: Duration::from_millis(2_500),
        packet_count: 7,
        bytes_written: 4_096,
        error: error.map(str::to_owned),
    }
}

/// The status a mod reads: the id, the time in ms and the bytes written; `failed` only for a
/// recorder that stopped on an error; empty when no recording runs.
#[test]
fn the_status_says_what_the_recorder_says() {
    let running = status_of(Some(&recorder_status(
        true,
        "recordings/trip_2026-11-22_10-00-00_123_4.mcpr",
        None,
    )));
    assert!(running.active && !running.failed);
    assert_eq!(running.recording, "trip_2026-11-22_10-00-00_123_4");
    assert_eq!((running.elapsed_ms, running.bytes), (2_500, 4_096));

    let failed = status_of(Some(&recorder_status(
        false,
        "recordings/trip.mcpr",
        Some("recorder queue is full; capture stopped"),
    )));
    assert!(!failed.active && failed.failed);
    assert_eq!(failed.recording, "trip");

    // A name no id can hold (`/studio record start` takes any word): no id, still a recording.
    let odd = status_of(Some(&recorder_status(true, "recordings/a:b_1.mcpr", None)));
    assert!(odd.active);
    assert_eq!(odd.recording, "");

    let none = status_of(None);
    assert!(!none.active && !none.failed);
    assert_eq!(
        (none.recording.as_str(), none.elapsed_ms, none.bytes),
        ("", 0, 0)
    );
}

/// A mod records the world session: nothing without one, one recording at a time, the status
/// naming the recording while it runs, and the archive in the recordings folder, under the id
/// `recording_stop` answers, once it ends.
#[test]
#[ignore = "daily"]
fn a_client_mod_records_the_session_into_the_recordings_folder() {
    let root = tempfile::tempdir().unwrap();
    crate::game::native_entity_refs::tests::with_client_game(|game| {
        game.replay.recordings_dir = Some(root.path().to_path_buf());
        assert_eq!(
            game.recording_start(Some("trip")),
            Err(RecordingError::NotConnected)
        );
        assert_eq!(game.recording_stop(), Err(RecordingError::NotConnected));
        assert!(!game.recording_status().active);

        game.init_remote_for_replay().unwrap();
        assert_eq!(game.recording_stop(), Err(RecordingError::NotRecording));
        let id = game.recording_start(Some("my trip")).unwrap();
        assert!(id.starts_with("my_trip_"), "{id}");
        let status = game.recording_status();
        assert!(status.active && !status.failed);
        assert_eq!(status.recording, id);
        assert_eq!(
            game.recording_start(None),
            Err(RecordingError::AlreadyRecording)
        );
        assert!(
            !root.path().join(format!("{id}.mcpr")).exists(),
            "the archive appears when the recording ends"
        );

        assert_eq!(game.recording_stop(), Ok(id.clone()));
        assert_eq!(
            recording_path(root.path(), &id),
            Ok(root.path().join(format!("{id}.mcpr")))
        );
        assert!(!game.recording_status().active);
        assert_eq!(game.recording_stop(), Err(RecordingError::NotRecording));
    });
}
