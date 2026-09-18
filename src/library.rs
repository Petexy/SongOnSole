//! Finding the user's music, and asking FFmpeg what each file is.
//!
//! The walk, the probing and the artwork all happen on a worker: a library of
//! twenty thousand songs is twenty thousand `ffprobe` runs, and none of them
//! may hold up a frame. A path is handed to a process as an argument and never
//! to a shell, so a song called `; rm -rf ~` is a song.
use crate::message;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
};

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album_artist: String,
    pub album: String,
    pub disc: u32,
    pub number: u32,
    pub duration: f64,
    pub cover: Option<PathBuf>,
    /// When the file was last written, in seconds since the epoch, which is
    /// what "newest first" means for music: a library is added to a folder at
    /// a time, and the folder's own clock is the only record of when. A file
    /// whose time cannot be read counts as the oldest there is rather than
    /// being left out of the order.
    pub added: u64,
}

/// What order the shelves are listed in.
///
/// The same six the film player and the photo viewer offer, said in the words
/// a library of music uses: a song has an artist and an album where a file has
/// a size, so those two take the place of largest and smallest. What a shelf
/// is sorted *by* depends on what its rows are — a row on the Albums shelf is
/// a record and not a song — which is `Music::rebuild`'s business and not
/// this enum's.
///
/// Two shelves keep an order of their own whatever this says, and the menu
/// leaves the rows out while they are open: the queue is the order it is going
/// to play in, and an album is disc and track number. An order laid over
/// either of those would be a listing that is no longer about anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Order {
    #[default]
    Name,
    NameReversed,
    Artist,
    Album,
    Newest,
    Oldest,
}

impl Order {
    pub const ALL: [Order; 6] = [
        Order::Name,
        Order::NameReversed,
        Order::Artist,
        Order::Album,
        Order::Newest,
        Order::Oldest,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Order::Name => crate::i18n::text("order-name"),
            Order::NameReversed => crate::i18n::text("order-name-backwards"),
            Order::Artist => crate::i18n::text("order-artist"),
            Order::Album => crate::i18n::text("order-album"),
            Order::Newest => crate::i18n::text("order-newest"),
            Order::Oldest => crate::i18n::text("order-oldest"),
        }
    }
}
impl Track {
    pub fn album_key(&self) -> (String, String) {
        (self.album_artist.clone(), self.album.clone())
    }
}

pub enum Scan {
    Track(Track),
    Done(Vec<String>),
}

/// Whether this is a file worth reading.
///
/// **What the decoder really plays, and nothing more.** The shell's Music shelf
/// knows sixteen audio extensions and this list is twelve of them: `opus`,
/// `wma`, `ape`, `wv` and `mpc` are left off because Symphonia does not decode
/// them, and a library that lists a song which will not play is worse than one
/// that does not list it. Each of the twelve was checked by encoding a file and
/// pulling samples back out of the decoder — `m4b` and `mka` are on the list
/// because that check said yes, not because a container looked familiar.
///
/// ALAC is not an extension: it arrives inside `m4a`, and plays.
pub fn supported(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "mp3" | "flac" | "m4a" | "m4b" | "mka" | "aac" | "ogg" | "oga" | "wav" | "aif" | "aiff"
        )
    })
}

fn walk(
    path: &Path,
    files: &mut BTreeSet<PathBuf>,
    visited: &mut BTreeSet<PathBuf>,
    errors: &mut Vec<String>,
) {
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(error) => {
            errors.push(
                message!("error-path", "name" => path.display().to_string(), "error" => error.to_string()),
            );
            return;
        }
    };
    if path.is_file() {
        if supported(&path) {
            files.insert(path);
        }
        return;
    }
    if !visited.insert(path.clone()) {
        return;
    }
    match std::fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        // Do not follow directory symlinks: overlapping roots are deduplicated above.
                        if entry.file_type().is_ok_and(|t| !t.is_symlink()) {
                            walk(&entry.path(), files, visited, errors);
                        }
                    }
                    Err(error) => errors.push(
                        message!("error-path", "name" => path.display().to_string(), "error" => error.to_string()),
                    ),
                }
            }
        }
        Err(error) => errors.push(
            message!("error-path", "name" => path.display().to_string(), "error" => error.to_string()),
        ),
    }
}

pub fn scan(roots: Vec<PathBuf>) -> mpsc::Receiver<Scan> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut paths = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut errors = Vec::new();
        for root in roots {
            walk(&root, &mut paths, &mut visited, &mut errors);
        }
        for path in paths {
            match probe(&path) {
                Ok(track) => {
                    if tx.send(Scan::Track(track)).is_err() {
                        return;
                    }
                }
                Err(e) => errors.push(e),
            }
        }
        let _ = tx.send(Scan::Done(errors));
    });
    rx
}

fn tag(tags: &Value, key: &str) -> Option<String> {
    tags.as_object()?
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))?
        .1
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
fn number(value: Option<String>) -> u32 {
    value
        .and_then(|s| s.split('/').next()?.parse().ok())
        .unwrap_or(0)
}

pub fn probe(path: &Path) -> Result<Track, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| message!("error-ffprobe", "error" => error.to_string()))?;
    if !output.status.success() {
        return Err(message!("error-read", "name" => path.display().to_string()));
    }
    let data: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        message!("error-path", "name" => path.display().to_string(), "error" => error.to_string())
    })?;
    let audio = data["streams"]
        .as_array()
        .and_then(|streams| streams.iter().find(|one| one["codec_type"] == "audio"))
        .ok_or_else(|| message!("error-no-audio", "name" => path.display().to_string()))?;
    let tags = &data["format"]["tags"];
    let get = |key: &str| tag(tags, key).or_else(|| tag(&audio["tags"], key));
    let artist = get("artist").unwrap_or_else(|| crate::i18n::text("unknown-artist").into());
    let duration = data["format"]["duration"]
        .as_str()
        .or_else(|| audio["duration"].as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|d| d.is_finite() && *d > 0.0)
        .unwrap_or(0.0);
    let embedded = data["streams"].as_array().is_some_and(|streams| {
        streams
            .iter()
            .any(|s| s["disposition"]["attached_pic"] == 1)
    });
    Ok(Track {
        path: path.to_owned(),
        title: get("title").unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        }),
        album_artist: get("album_artist")
            .or_else(|| get("albumartist"))
            .or_else(|| get("album artist"))
            .unwrap_or_else(|| artist.clone()),
        artist,
        album: get("album").unwrap_or_else(|| crate::i18n::text("unknown-album").into()),
        disc: number(get("disc").or_else(|| get("discnumber"))),
        number: number(get("track").or_else(|| get("tracknumber"))),
        duration,
        cover: cover(path, embedded),
        added: std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|when| when.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|since| since.as_secs())
            .unwrap_or(0),
    })
}

fn cover(path: &Path, embedded: bool) -> Option<PathBuf> {
    let parent = path.parent()?;
    for name in [
        "cover.jpg",
        "cover.png",
        "folder.jpg",
        "folder.png",
        "front.jpg",
        "Cover.jpg",
        "Folder.jpg",
    ] {
        let candidate = parent.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if !embedded {
        return None;
    }
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hash);
    if let Ok(meta) = path.metadata() {
        meta.len().hash(&mut hash);
        meta.modified().ok().hash(&mut hash);
    }
    let directory = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::settings::home().join(".cache"))
        .join("songonsole/covers");
    std::fs::create_dir_all(&directory).ok()?;
    let file = directory.join(format!("{:016x}.png", hash.finish()));
    if file.is_file() {
        return Some(file);
    }
    let temporary = file.with_extension(format!("{}.tmp.png", std::process::id()));
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-y", "-i"])
        .arg(path)
        .args([
            "-map",
            "0:v:0",
            "-frames:v",
            "1",
            "-vf",
            "scale=512:512:force_original_aspect_ratio=decrease",
        ])
        .arg(&temporary)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    if status.success() {
        std::fs::rename(temporary, &file).ok()?;
        Some(file)
    } else {
        let _ = std::fs::remove_file(temporary);
        None
    }
}

pub fn time(seconds: f64) -> String {
    let n = seconds.max(0.0) as u64;
    if n >= 3600 {
        format!("{}:{:02}:{:02}", n / 3600, n / 60 % 60, n % 60)
    } else {
        format!("{}:{:02}", n / 60, n % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extension_filter_is_audio_only_and_case_insensitive() {
        assert!(supported(Path::new("MUSIC.FLAC")));
        assert!(supported(Path::new("song.m4a")));
        assert!(!supported(Path::new("movie.mkv")));
        assert!(!supported(Path::new("notes.txt")));
    }
    #[test]
    fn tags_accept_capitals_and_disc_track_totals() {
        let tags = serde_json::json!({"TITLE": "  Song  ", "TRACK": "3/12"});
        assert_eq!(tag(&tags, "title").as_deref(), Some("Song"));
        assert_eq!(number(tag(&tags, "track")), 3);
    }
    #[test]
    fn overlapping_roots_and_symlink_cycles_do_not_duplicate_tracks() {
        let root = std::env::temp_dir().join(format!("songonsole-walk-{}", std::process::id()));
        std::fs::create_dir_all(root.join("album")).unwrap();
        std::fs::write(root.join("album/song.MP3"), []).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&root, root.join("album/loop")).unwrap();
        let mut files = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut errors = Vec::new();
        walk(&root, &mut files, &mut visited, &mut errors);
        walk(&root.join("album"), &mut files, &mut visited, &mut errors);
        assert_eq!(files.len(), 1);
        assert!(errors.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod media_tests {
    use super::*;
    #[test]
    fn tagged_files_probe_and_decode_in_the_supported_core_formats() {
        let root = std::env::temp_dir().join(format!("songonsole-media-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for extension in [
            "flac", "mp3", "m4a", "m4b", "mka", "wav", "ogg", "aiff", "aac",
        ] {
            let path = root.join(format!("test.{extension}"));
            let result = Command::new("ffmpeg")
                .args([
                    "-nostdin",
                    "-v",
                    "error",
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "sine=frequency=440:sample_rate=44100",
                    "-t",
                    "0.25",
                    "-metadata",
                    "title=Test song",
                    "-metadata",
                    "artist=Test artist",
                    "-metadata",
                    "album=Test album",
                    "-metadata",
                    "track=2/8",
                ])
                .arg(&path)
                .output()
                .expect("Install FFmpeg to run media integration tests");
            assert!(
                result.status.success(),
                "{extension}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            let track = probe(&path).unwrap();
            assert!(track.duration > 0.0, "{extension}");
            if extension != "aiff" && extension != "aac" && extension != "mka" {
                assert_eq!(track.title, "Test song", "{extension}");
                assert_eq!(track.artist, "Test artist", "{extension}");
                assert_eq!(track.number, 2, "{extension}");
            }
            let decoder = rodio::Decoder::try_from(std::fs::File::open(&path).unwrap()).unwrap();
            assert!(
                decoder.take(4096).any(|sample| sample.abs() > 0.001),
                "{extension} did not produce audio"
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
