//! Music: a library and a player for somebody sitting across the room.
//!
//! **Music** to a person, `songonsole` to the machine — the same split the
//! photo viewer and the film player have, and the same one the software hub
//! has. The name in the title bar, in the shell and on the desktop entry is
//! the translated one; the executable, the `StartupWMClass`, the package and
//! the AppStream id are all `songonsole` and never anything else.
mod app;
mod audio;
mod draw;
mod i18n;
mod legend;
mod library;
mod pad;
mod playing;
mod queue;
mod settings;

use lxb_toolkit::input::Action;
use std::path::PathBuf;

/// The machine's name for this program: the executable, the window class, the
/// package and the id its desktop entry is filed under. `packaging/check.sh`
/// reads this line.
const APP_ID: &str = "songonsole";

const HELP: &str = "\
Music — your library, from across the room.

Usage: songonsole [FOLDER|SONG] [OPTIONS]

  --shot FILE       write one settled frame to a PNG and stop
  --size WxH        how large that frame is (default 1280x800)
  --width N, --height N   the same, said the way the other three say it
  --view NAME       open on songs, albums, artists, favourites or queue
  --playing         open the Now Playing page
  --after SECONDS   how long after the press the picture is taken
  --back            press Back rather than opening, so the way out can be seen
  --demo            a made-up library, for pictures; nothing of yours is touched
  --controllers     list what this machine can be driven with
  --version         print the version
  --help            print this message

  D-pad or arrows   move            South or Enter   choose
  East or Escape    back            North or F10     options
  Bumpers or Tab    change shelf    Start            play or pause

On a bar — how far through the song, and how loud — left and right move the bar
itself. The stick moves it by how far it is pushed, so a thumb just off centre
creeps and a thumb at the stop crosses the song. A pointer sets either bar by
landing on it, and goes on setting it while the button is held.

Music keeps playing while you browse. FFmpeg reads the tags and the artwork.";

fn main() {
    if let Err(error) = run() {
        eprintln!("songonsole: {error}");
        std::process::exit(1);
    }
}

/// What this machine can be driven with, and what was looked at.
///
/// Exists because "the controller does nothing" has two completely different
/// causes — no reader at all, and a reader that found no gamepad — and from
/// the outside they look identical. The film player and the photo viewer each
/// answer the same question the same way.
fn controllers() {
    let pad = pad::Pad::new();
    if let Some(trouble) = pad.trouble() {
        println!("No controller support at all: {trouble}");
        return;
    }
    let found = pad.found();
    if found.is_empty() {
        println!("No controllers found.");
        println!();
        println!("Nothing on this machine is presenting a gamepad. That is not");
        println!("always a fault: a controller whose driver is not in the kernel");
        println!("— a Steam Controller outside the session shell that drives it,");
        println!("for one — appears as a mouse and a keyboard and no gamepad, so");
        println!("there is nothing here for any program to read.");
        println!();
        println!("Look for one with:  ls /dev/input/js*");
        return;
    }
    println!(
        "{} controller{} found:",
        found.len(),
        if found.len() == 1 { "" } else { "s" }
    );
    for one in &found {
        println!();
        println!("  {}", one.name);
        println!(
            "    buttons   {}",
            if one.mapped {
                "named by SDL's mapping list"
            } else {
                "guessed from the driver's own numbering"
            }
        );
        println!(
            "    stick     {}",
            if one.stick {
                "read, and moves a bar by how far it is pushed"
            } else {
                "none reported; bars move by ten seconds a press"
            }
        );
    }
}

/// A window this program would really open. The floor is the size the layout
/// is built down to; the ceiling is there so that a typo cannot ask for a
/// texture no card will allocate.
fn bound(size: (u32, u32)) -> Result<(), String> {
    if size.0 < 640 || size.1 < 400 || size.0 > 7680 || size.1 > 4320 {
        return Err("Size must be between 640x400 and 7680x4320".into());
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut opening = None;
    let mut shot = None;
    let mut demo = false;
    let mut view = 0;
    let mut size = (1280, 800);
    let mut playing = false;
    let mut back = false;
    let mut after: Option<f32> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{HELP}");
                return Ok(());
            }
            "--version" => {
                println!("{APP_ID} {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--controllers" => {
                controllers();
                return Ok(());
            }
            "--shot" => shot = Some(args.next().ok_or("--shot needs a PNG path")?),
            "--demo" => demo = true,
            "--playing" => playing = true,
            "--back" => back = true,
            "--after" => {
                let value = args.next().ok_or("--after needs a number of seconds")?;
                let seconds: f32 = value.parse().map_err(|_| "--after needs a number")?;
                if !seconds.is_finite() || !(0.0..=60.0).contains(&seconds) {
                    return Err("--after must be between 0 and 60 seconds".into());
                }
                after = Some(seconds);
            }
            "--view" => {
                let name = args
                    .next()
                    .ok_or("--view needs songs, albums, artists, favourites or queue")?;
                view = app::TABS
                    .iter()
                    .position(|tab| *tab == name)
                    .ok_or("Unknown library view")?;
            }
            "--size" => {
                let value = args.next().ok_or("--size needs WIDTHxHEIGHT")?;
                let (w, h) = value.split_once('x').ok_or("--size needs WIDTHxHEIGHT")?;
                size = (
                    w.parse::<u32>().map_err(|_| "Invalid width")?,
                    h.parse::<u32>().map_err(|_| "Invalid height")?,
                );
                bound(size)?;
            }
            // The two the other three applications take, so that one line in a
            // README photographs any of them. `--size` stays: it is what every
            // picture in this repository was taken with.
            "--width" => {
                size.0 = args
                    .next()
                    .ok_or("--width needs a number")?
                    .parse::<u32>()
                    .map_err(|_| "Invalid width")?;
                bound(size)?;
            }
            "--height" => {
                size.1 = args
                    .next()
                    .ok_or("--height needs a number")?
                    .parse::<u32>()
                    .map_err(|_| "Invalid height")?;
                bound(size)?;
            }
            option if option.starts_with('-') => return Err(format!("Unknown option: {option}")),
            path => {
                if opening.is_some() {
                    return Err("Pass one folder or song".into());
                }
                let path = PathBuf::from(path)
                    .canonicalize()
                    .map_err(|error| format!("{path}: {error}"))?;
                if path.is_file() && !library::supported(&path) {
                    return Err("Choose a supported music file or folder".into());
                }
                opening = Some(path);
            }
        }
    }

    let mut state = app::Music::new(opening, demo, shot.is_some());
    // **No `own_file_questions`.** That would force the toolkit's built-in
    // picker and never ask the desktop — which is why this used to raise a
    // chooser neither Pictures nor Videos has ever shown. Those two ask the
    // desktop first, which on LineXinBar is the shell's own chooser drawn as
    // the bar, and fall back to the built-in one only where there is no
    // portal. One file question for the whole family.
    let window = lxb_app::App::new(APP_ID, i18n::text("app-name"))
        .plain()
        .driven();
    if let Some(path) = shot {
        state.finish_scan();
        state.change_tab(view);
        // A press has to wait until a page has been laid out: only drawing
        // knows where the cards went, and a page grows out of a rectangle. A
        // shot with no `--after` is a picture of a page that has arrived, so
        // the default is long enough ago that nothing is still moving.
        state.defer(
            (playing || back).then_some(if back { Action::Back } else { Action::Accept }),
            after.unwrap_or(1.0e6),
        );
        window.shot(path, size.0, size.1, 10.0, |page| state.frame(page))
    } else {
        state.change_tab(view);
        window.run(move |page| state.frame(page))
    }
}
