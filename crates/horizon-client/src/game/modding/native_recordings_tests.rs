//! A client mod lists the saved recordings (#19245).
use super::native_recordings::list_recordings;
use horizon_client::studio::{McprWriter, Metadata};
use std::path::Path;

/// A recording `/studio record` could have written: one packet, with the metadata given.
fn write_recording(path: &Path, date: u64, duration: u32, server: &str, singleplayer: bool) {
    let meta = Metadata {
        date,
        duration,
        server_name: server.to_owned(),
        singleplayer,
        mcversion: "26.3".to_owned(),
        ..Default::default()
    };
    let mut writer = McprWriter::new(path.to_path_buf(), meta).unwrap();
    writer.write_packet(0, 0x01, &[1]).unwrap();
    writer.finish(duration).unwrap();
}

/// The list is what `replay_open` can open, newest first, with the archive's own metadata; a file
/// that is no readable archive is still listed from what the file system says, and nothing an id
/// cannot name (or that is no regular file) is listed.
#[test]
fn the_list_holds_the_recordings_an_id_can_open_newest_first() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path();
    write_recording(
        &dir.join("old_trip.mcpr"),
        2_000,
        5_000,
        "play.example.org",
        false,
    );
    write_recording(&dir.join("new_trip.mcpr"), 3_000, 9_500, "local", true);
    // Not an archive: listed, dated by its last change (set to 1 s after the epoch).
    let broken = dir.join("broken.mcpr");
    std::fs::write(&broken, b"not a zip").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&broken)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_millis(1_000))
        .unwrap();
    // Left out: other kinds of file, a folder named like a recording, a name no id can hold.
    std::fs::write(dir.join("notes.txt"), b"x").unwrap();
    std::fs::write(dir.join("trip.mcpr.tmp"), b"x").unwrap();
    std::fs::create_dir(dir.join("folder.mcpr")).unwrap();
    std::fs::write(dir.join("a:b.mcpr"), b"x").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(dir.join("new_trip.mcpr"), dir.join("linked.mcpr")).unwrap();

    let listed = list_recordings(dir);
    let ids: Vec<&str> = listed.iter().map(|info| info.id.as_str()).collect();
    assert_eq!(ids, ["new_trip", "old_trip", "broken"]);

    let new = &listed[0];
    assert_eq!(new.name, "new_trip");
    assert_eq!((new.duration_ms, new.date), (9_500, 3_000));
    assert_eq!(
        new.size,
        std::fs::metadata(dir.join("new_trip.mcpr")).unwrap().len()
    );
    assert_eq!(
        (new.server.as_str(), new.version.as_str()),
        ("local", "26.3")
    );
    assert!(new.singleplayer);

    let old = &listed[1];
    assert_eq!((old.duration_ms, old.date), (5_000, 2_000));
    assert_eq!(old.server, "play.example.org");
    assert!(!old.singleplayer);

    let broken = &listed[2];
    assert_eq!(
        (broken.duration_ms, broken.date, broken.size),
        (0, 1_000, 9)
    );
    assert_eq!((broken.server.as_str(), broken.version.as_str()), ("", ""));
}

/// Recordings of one date list by id, and a missing folder lists nothing.
#[test]
fn equal_dates_list_by_id_and_a_missing_folder_is_empty() {
    let root = tempfile::tempdir().unwrap();
    for id in ["b", "c", "a"] {
        write_recording(&root.path().join(format!("{id}.mcpr")), 7, 1, "local", true);
    }
    let ids: Vec<String> = list_recordings(root.path())
        .into_iter()
        .map(|info| info.id)
        .collect();
    assert_eq!(ids, ["a", "b", "c"]);
    assert!(list_recordings(&root.path().join("missing")).is_empty());
}
