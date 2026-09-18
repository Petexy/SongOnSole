//! What Music is, and what a press does to it.
//!
//! Every number that moves on the screen is worked out here from **the scene
//! clock**, `Page::seconds`, and never accumulated frame by frame. That is not
//! tidiness: `--shot` draws two frames at one instant, so a `dt` of nought
//! would freeze every animation at whatever it happened to be, and a picture
//! of an animation would be a picture of nothing. An animation that knows when
//! it began can be asked what it looks like at any moment, which is also what
//! `--after` is.
use crate::{
    audio::{Audio, Command},
    i18n,
    library::{self, Order, Scan, Track},
    message,
    queue::Queue,
    settings::Settings,
};
use lxb_app::{Page, PickerSelection};
use lxb_toolkit::{
    input::Action,
    motion::{self, duration},
    sound::Sound,
};
use std::{collections::BTreeMap, path::PathBuf, sync::mpsc};

/// The rail down the left-hand side, in order. The identifiers are also the
/// message ids the rail is lettered with, and what `--view` is given.
pub const TABS: [&str; 5] = ["songs", "albums", "artists", "favourites", "queue"];

/// What the rail down the left-hand side carries, in the order a direction
/// walks it: the five shelves, and then the two buttons at the foot of it.
///
/// **The two buttons are rows of the rail, not decoration on it.** They were
/// drawn there and numbered for a pointer and nothing else, so on a machine
/// with a controller and no mouse there was no way to press either of them —
/// which is the same fault the record at the head of the strip had, in the
/// same place, for the same reason.
pub const RAIL_ADD_FOLDER: usize = TABS.len();
pub const RAIL_OPTIONS: usize = TABS.len() + 1;
pub const RAIL_ROWS: usize = TABS.len() + 2;

/// What the transport carries, in the order a direction walks it: the two
/// bars, and then the five buttons.
///
/// **A bar is a thing you stand on, not a pair of buttons beside it.** Moving
/// through a song and setting the volume were four chips in the row — back ten
/// seconds, on ten seconds, quieter, louder — which is a way of saying that
/// the bar drawn above them was a picture and not a control. It is the control
/// now, the way every bar in the shell is one: the light stands on the whole
/// row, Left and Right move the handle along it, and a pointer landing
/// anywhere on the groove says a place outright. The four chips are gone; five
/// are left, and every one of them does something no direction could.
///
/// Up and Down are what get on and off a bar, because Left and Right are
/// spoken for while the light is on one — the same shape the Settings column
/// has, where a row is reached down the column and moved across it.
///
/// **The record at the head of the strip is one of them.** It says what is
/// playing and it opens the Now Playing page, and until 2026-09-19 the only
/// way to reach it was to click on it: a way back to the song you are
/// listening to that a controller cannot take is no way back at all on a
/// console. It is the top of the walk, above the two bars, which is where it
/// is drawn. The Now Playing page has no such row — it *is* that page — so the
/// walk there starts at the position bar; see [`Music::strip_top`].
///
/// One model on both screens: the same order, the same index, the same
/// presses, drawn small under the library and large under the sleeve.
pub const NOW_PLAYING: usize = 0;
pub const BAR_POSITION: usize = 1;
pub const BAR_LOUDNESS: usize = 2;
pub const FIRST_BUTTON: usize = 3;
pub const TRANSPORT: usize = 8;

/// What a row is sorted by, whichever shelf it is on.
///
/// Gathered once and compared many times rather than read off the track on
/// every comparison: `sort_by_key` would want a fresh `String` per comparison,
/// and a library of a hundred thousand songs is a great many fresh strings.
struct SortKey {
    name: String,
    artist: String,
    album: String,
    added: u64,
}

/// Put the rows in the order that was asked for.
///
/// Every order falls back to the name, so two records by the same artist and
/// two songs added in the same second are still in an order somebody can
/// predict rather than in whatever order the folder was read in.
fn sort_rows<T>(rows: &mut [(SortKey, T)], order: Order) {
    match order {
        Order::Name => rows.sort_by(|a, b| a.0.name.cmp(&b.0.name)),
        Order::NameReversed => rows.sort_by(|a, b| b.0.name.cmp(&a.0.name)),
        Order::Artist => {
            rows.sort_by(|a, b| a.0.artist.cmp(&b.0.artist).then(a.0.name.cmp(&b.0.name)))
        }
        Order::Album => {
            rows.sort_by(|a, b| a.0.album.cmp(&b.0.album).then(a.0.name.cmp(&b.0.name)))
        }
        Order::Newest => {
            rows.sort_by(|a, b| b.0.added.cmp(&a.0.added).then(a.0.name.cmp(&b.0.name)))
        }
        Order::Oldest => {
            rows.sort_by(|a, b| a.0.added.cmp(&b.0.added).then(a.0.name.cmp(&b.0.name)))
        }
    }
}

/// A line in the browser: one song, or one album or artist standing for
/// several.
#[derive(Clone)]
pub struct Row {
    pub label: String,
    pub detail: String,
    pub tracks: Vec<usize>,
    pub cover: Option<PathBuf>,
    /// Which occurrence in the queue this row is, where the queue is what is
    /// being shown. A song added twice is two rows, and taking one out must
    /// take out the one that was pressed.
    pub queue_at: Option<usize>,
}

/// Where drawing put a row, so that a press can name a rectangle.
///
/// Only drawing knows, and it is written down again every frame the browser is
/// drawn — a window resized while a page is open still shrinks back into the
/// right card. The picture is kept apart from the whole row because a page
/// grows out of the *sleeve*, and a card's caption is a jump on the first
/// frame otherwise.
#[derive(Clone, Copy)]
pub struct Card {
    pub index: usize,
    pub rect: [f32; 4],
    pub art: [f32; 4],
}

impl Card {
    /// A card nothing has drawn. A page opening out of it grows out of a point
    /// rather than out of a rectangle, which is what the very first frame of a
    /// session would otherwise have to invent.
    pub const fn nowhere() -> Self {
        Self {
            index: usize::MAX,
            rect: [0.0; 4],
            art: [0.0; 4],
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::nowhere()
    }
}

/// The wall of albums or artists, opened at one of them.
#[derive(Clone)]
pub struct Group {
    pub title: String,
    pub subtitle: String,
    pub tracks: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Sidebar,
    Library,
    Transport,
    /// The up-next list down the right-hand side of the Now Playing page.
    Queue,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Library,
    Playing,
}

/// Which two pages a crossing is between.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// A wall of albums or artists opening into the songs on one of them.
    Group,
    /// Anything at all opening into the Now Playing page.
    Playing,
}

/// One page growing out of the card that was pressed, and shrinking back into
/// it.
///
/// One number drives the whole of it — where the page is, how round its
/// corners are, how far the page behind has stepped back and faded, and how
/// solid the words underneath are cut. Two clocks would land the sleeve and
/// its own chrome on different frames.
pub struct Crossing {
    pub what: Change,
    pub back: bool,
    /// The card it grows out of, and the picture inside that card.
    pub from: [f32; 4],
    pub art: [f32; 4],
    began: f32,
    /// The page being left, kept alive until it has landed: going back out of
    /// an album, the state has already become the wall of albums, and the list
    /// of songs shrinking into a card has to be drawn from somewhere.
    pub leaving: Vec<Row>,
    pub title: String,
    pub subtitle: String,
}

impl Crossing {
    /// How far along it is, nought to one, before any easing.
    fn travelled(&self, seconds: f32) -> f32 {
        ((seconds - self.began) / duration::LAUNCH_OPEN).clamp(0.0, 1.0)
    }

    /// Nought on the card, one filling the window.
    pub fn grown(&self, seconds: f32) -> f32 {
        let travelled = self.travelled(seconds);
        motion::ease(if self.back {
            1.0 - travelled
        } else {
            travelled
        })
    }

    /// How solid the page standing over the other one really is.
    ///
    /// Faster than the rectangle on purpose: most of a crossing is then one
    /// page growing over a cleared page, rather than two pages at half
    /// strength each with the wallpaper showing through the middle.
    pub fn arriving(&self, seconds: f32) -> f32 {
        (self.grown(seconds) * 3.0).min(1.0)
    }

    fn landed(&self, seconds: f32) -> bool {
        self.travelled(seconds) >= 1.0
    }
}

/// A press to make once the page has been laid out, which is the only way to
/// photograph an animation: settle the page, press, and ask what it looks like
/// a given number of seconds later.
pub struct Deferred {
    pub press: Option<Action>,
    pub after: f32,
}

/// A bar being moved, and how long it is still believed to be held.
///
/// Both bars are moved the same way and for the same reason: what reaches them
/// is a *rate* — a thumb leaning on a stick, a pointer dragged along the
/// groove, a direction held down — and neither the decoder nor the settings
/// file can be asked to keep up with sixty of those a second. So the thumb's
/// own idea of where the bar is lives here, the dear half of it happens on a
/// cadence, and letting go is what makes it final.
///
/// It is also what the interface *shows* while it is held. Without it the
/// groove would snap back between seeks to wherever the sound worker last
/// reported, which is a bar that shudders under the thumb moving it.
struct Drive {
    /// [`BAR_POSITION`] or [`BAR_LOUDNESS`]: which bar this is.
    what: usize,
    /// Where the thumb has it — seconds into the song, or nought to one.
    to: f64,
    /// How long since the worker was told, for the one of the two that is
    /// dear to tell.
    told: f32,
    /// How long this is still believed to be held. A stick and a pointer both
    /// say every frame that they are still there; a held direction arrives as
    /// one press and then as repeats of it, which is a hold with gaps in.
    left: f32,
}

/// How fast a stick at its stop takes the song, in seconds of song per second.
///
/// Slower than the film player's triggers — a song is three minutes where a
/// film is two hours — and squared on the way in, so that the useful half of
/// the stick's travel is the slow half. See [`Music::drive_of`].
const SEEK_RATE: f64 = 45.0;

/// And how fast it takes the loudness, in whole range per second: a little
/// over a second from silent to full, at the stop.
const LOUD_RATE: f32 = 0.9;

/// How far the stick has to be pushed before it is being pushed at all.
///
/// A thumb resting on the stick and a worn spring both sit a little off
/// nought, and a song that crept away from where it was left whenever nobody
/// was touching it would be the most annoying possible bug.
const STICK_DEAD: f32 = 0.15;

/// How often a drive really seeks.
///
/// Every frame would be sixty seeks a second, each flushing a decoder; much
/// less often than this and the sound stops keeping up with the thumb.
const SEEK_EVERY: f32 = 0.10;

/// How long a drive is believed after the last push or press arrived.
///
/// Longer than the toolkit's own repeat interval, so that a held direction
/// reads as one drive rather than as a string of separate ones, and short
/// enough that letting go has written the settings file before the hand is
/// back.
const DRIVE_HELD: f32 = 0.25;

pub struct Music {
    pub settings: Settings,
    pub tracks: Vec<Track>,
    pub album_count: usize,
    pub queue: Queue,
    pub audio: Audio,
    pub tab: usize,
    pub selected: usize,
    /// Where the light is in the Now Playing page's up-next list.
    pub queued: usize,
    pub zone: Zone,
    pub screen: Screen,
    pub crossing: Option<Crossing>,
    pub light: lxb_render::Selection,
    pub transport: usize,
    /// Which row of the rail the light is on. Not the same as `tab`: the two
    /// buttons at the foot of the rail are rows the light can stand on and
    /// neither of them is a shelf, so standing on one does not change which
    /// shelf is open.
    pub rail: usize,
    /// Which zone handed the light to the strip, so that Up off the top of the
    /// strip undoes the Down that brought it there. Without it the light came
    /// off the strip into whichever zone was written here last, which is the
    /// one thing a direction must never be.
    pub strip_from: Zone,
    pub group: Option<Group>,
    pub rows: Vec<Row>,
    /// Where drawing put the browser's rows, in the order it drew them. The
    /// place in this list is the control's number, so a library of a hundred
    /// thousand songs numbers no more controls than a library of twenty.
    pub cards: Vec<Card>,
    /// Where drawing put the Now Playing page's up-next rows, on the same
    /// terms. `index` is the place in the queue rather than in the browser.
    pub queue_cards: Vec<Card>,
    /// Where drawing put the library's now-playing strip, which is the sleeve
    /// the page grows out of when the strip itself is pressed.
    pub strip: Card,
    /// And where it put the volume groove, because a click on a groove says a
    /// level and only the rectangle it was drawn in can turn a point into one.
    pub volume_track: [f32; 4],
    /// And the position groove, read the same way.
    pub position_track: [f32; 4],
    /// The loudness to come back to when the sound is turned on again.
    pub muted_from: f32,
    /// The pad's own reader, for the one thing a list of actions cannot carry:
    /// how far the stick is pushed. See [`crate::pad`].
    pad: crate::pad::Pad,
    /// The bar somebody is moving, if one is being moved. See [`Drive`].
    drive: Option<Drive>,
    pub scanning: bool,
    pub notice: String,
    pub demo: bool,
    pub visible_rows: usize,
    pub columns: usize,
    /// The scene clock, read at the top of every frame. Everything that moves
    /// is a function of this and of when it began.
    pub seconds: f32,
    /// How long the last frame took. Everything else on this page is a
    /// function of `seconds` and needs no such thing; a drive is the one part
    /// of this application that accumulates, and a rate is only a distance
    /// once it has been given a time.
    dt: f32,
    /// When the page now showing arrived, which is what the rows come in one
    /// after another from.
    pub arrived: f32,
    /// When the window itself arrived. The rail reads this rather than
    /// `arrived`, because the rail is the frame and not the page: it comes in
    /// once, with the application, and is still from then on.
    ///
    /// Nought, and never written again — the scene clock starts at nought in a
    /// window and stands at whatever `--shot` was given, so a live rail comes
    /// in and a photographed one is already there, with nothing told to settle.
    pub opened: f32,
    /// When the sleeve last changed, which is what it crosses over.
    pub sleeve_at: f32,
    /// When a press last landed, which is what makes the control under the
    /// light dip and come back.
    pressed_at: f32,
    pub sleeve_was: Option<PathBuf>,
    receiver: Option<mpsc::Receiver<Scan>>,
    opening: Option<PathBuf>,
    read_only: bool,
    ended_handled: bool,
    menu: Vec<Menu>,
    deferred: Option<Deferred>,
    frames: u32,
}

#[derive(Clone, Copy)]
enum Menu {
    /// The row that opens the orders, rather than one of them.
    Sorting,
    SortBy(Order),
    Play,
    Enqueue,
    Favourite,
    Unfavourite,
    Remove,
    AddFolder,
    Refresh,
    Clear,
}

/// Where a row stands while the page it is on is still coming in.
///
/// Nought while it is still waiting its turn and one once it has arrived; the
/// rows nearest the top go first, and a row far enough down the page arrives
/// after the last of them. Read as a fade and as a lift at once, so that one
/// number is the whole of a row's entrance.
pub fn entry(since: f32, index: usize) -> f32 {
    let waited = duration::ENTRY_LEAD + index as f32 * duration::ENTRY_STAGGER;
    motion::ease(((since - waited) / duration::ENTRY_SLIDE).clamp(0.0, 1.0))
}

impl Music {
    pub fn new(opening: Option<PathBuf>, demo: bool, read_only: bool) -> Self {
        let mut settings = if demo {
            Settings::default()
        } else {
            Settings::load()
        };
        if let Some(path) = &opening {
            let root = if path.is_dir() {
                path.clone()
            } else {
                path.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            };
            if !settings.folders.contains(&root) {
                settings.folders.push(root);
            }
        }
        let mut queue = Queue::default();
        queue.shuffle = settings.shuffle;
        queue.repeat = settings.repeat;
        let mut audio = Audio::new();
        audio.send(Command::Volume(settings.volume));
        let mut state = Self {
            settings,
            tracks: Vec::new(),
            album_count: 0,
            queue,
            audio,
            tab: 0,
            selected: 0,
            queued: 0,
            zone: Zone::Library,
            screen: Screen::Library,
            crossing: None,
            light: lxb_render::Selection::default(),
            // Play, which is the control a hand reaching for the strip is
            // reaching for. The two bars are a row above it.
            transport: FIRST_BUTTON + 3,
            rail: 0,
            strip_from: Zone::Library,
            group: None,
            rows: Vec::new(),
            cards: Vec::new(),
            queue_cards: Vec::new(),
            strip: Card::nowhere(),
            volume_track: [0.0; 4],
            position_track: [0.0; 4],
            muted_from: 0.0,
            pad: crate::pad::Pad::new(),
            drive: None,
            scanning: false,
            notice: String::new(),
            demo,
            visible_rows: 7,
            columns: 1,
            seconds: 0.0,
            dt: 0.0,
            arrived: 0.0,
            opened: 0.0,
            sleeve_at: 0.0,
            pressed_at: f32::MIN,
            sleeve_was: None,
            receiver: None,
            opening,
            read_only: read_only || demo,
            ended_handled: false,
            menu: Vec::new(),
            deferred: None,
            frames: 0,
        };
        if demo {
            state.demo_library();
        } else {
            state.scan();
        }
        state
    }

    /// What `--shot` asks for: a press to make once the page has been laid
    /// out, and how long after it the picture is taken.
    pub fn defer(&mut self, press: Option<Action>, after: f32) {
        self.deferred = Some(Deferred { press, after });
    }

    pub fn scan(&mut self) {
        // A refresh changes indices, so stop the old queue before replacing the library.
        self.audio.send(Command::Stop);
        self.queue.replace(Vec::new(), 0);
        self.tracks.clear();
        self.album_count = 0;
        self.rows.clear();
        self.group = None;
        self.selected = 0;
        self.scanning = true;
        self.notice.clear();
        self.receiver = Some(library::scan(self.settings.folders.clone()));
    }

    fn scan_event(&mut self, event: Scan) {
        match event {
            Scan::Track(track) => self.tracks.push(track),
            Scan::Done(errors) => {
                self.scanning = false;
                self.notice = match errors.split_first() {
                    None => String::new(),
                    Some((first, [])) => first.clone(),
                    Some((first, rest)) => {
                        message!("error-and-more", "error" => first.clone(), "count" => rest.len())
                    }
                };
                if let Some(path) = self.opening.take() {
                    if !self.read_only {
                        if let Some(index) = self.tracks.iter().position(|track| track.path == path)
                        {
                            self.queue.replace(vec![index], 0);
                            self.start_current();
                        }
                    }
                }
            }
        }
    }

    pub fn finish_scan(&mut self) {
        if let Some(receiver) = self.receiver.take() {
            for event in receiver {
                self.scan_event(event);
            }
        }
        self.rebuild();
    }

    pub fn save(&mut self) {
        if !self.read_only {
            if let Err(error) = self.settings.save() {
                self.notice = message!("error-save", "error" => error);
            }
        }
    }

    pub fn current(&self) -> Option<&Track> {
        self.queue.current().and_then(|i| self.tracks.get(i))
    }

    /// How far through its dip the control under the light is, or `None` where
    /// nothing has been pressed lately.
    ///
    /// `lxb-app` winds its own `Pressing` on for a *pointer* click and not for
    /// a pad or a key: a driven page is handed the action instead and decides
    /// what it meant, so the press it makes has to be its own. The curve is the
    /// toolkit's — `motion::press_scale` over `duration::GUIDE_PRESS` — because
    /// a second way of dipping a button is a second design language.
    pub fn press_through(&self) -> Option<f32> {
        let through = (self.seconds - self.pressed_at) / duration::GUIDE_PRESS;
        (0.0..1.0).contains(&through).then_some(through)
    }

    /// The sleeve of what is playing, if it has one.
    pub fn sleeve(&self) -> Option<&std::path::Path> {
        self.current().and_then(|track| track.cover.as_deref())
    }

    /// A wall of albums rather than a list of songs.
    pub fn grid(&self) -> bool {
        self.tab == 1 && self.group.is_none()
    }

    pub fn title(&self) -> String {
        self.group
            .as_ref()
            .map(|group| group.title.clone())
            .unwrap_or_else(|| i18n::text(TABS[self.tab]).into())
    }

    pub fn subtitle(&self) -> String {
        if let Some(group) = &self.group {
            return group.subtitle.clone();
        }
        match self.tab {
            1 => i18n::text("albums-subtitle").into(),
            2 => i18n::text("artists-subtitle").into(),
            3 => i18n::text("favourites-subtitle").into(),
            4 => i18n::text("queue-subtitle").into(),
            _ => i18n::text("songs-subtitle").into(),
        }
    }

    pub fn rebuild(&mut self) {
        self.album_count = self
            .tracks
            .iter()
            .map(|track| track.album_key())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let indices: Vec<usize> = if let Some(group) = &self.group {
            group.tracks.clone()
        } else if self.tab == 4 {
            self.queue.entries.clone()
        } else {
            (0..self.tracks.len())
                .filter(|i| {
                    self.tab != 3 || self.settings.favourites.contains(&self.tracks[*i].path)
                })
                .collect()
        };
        if self.group.is_none() && (self.tab == 1 || self.tab == 2) {
            let mut groups: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
            for index in indices {
                let track = &self.tracks[index];
                let key = if self.tab == 1 {
                    track.album_key()
                } else {
                    (track.artist.clone(), String::new())
                };
                groups.entry(key).or_default().push(index);
            }
            self.rows = groups
                .into_iter()
                .map(|((artist, album), mut tracks)| {
                    tracks.sort_by_key(|i| {
                        let t = &self.tracks[*i];
                        (
                            t.album.to_lowercase(),
                            t.disc,
                            t.number,
                            t.title.to_lowercase(),
                        )
                    });
                    let cover = tracks.iter().find_map(|i| self.tracks[*i].cover.clone());
                    Row {
                        label: if self.tab == 1 { album } else { artist.clone() },
                        detail: if self.tab == 1 {
                            artist
                        } else {
                            message!("song-count", "count" => tracks.len())
                        },
                        tracks,
                        cover,
                        queue_at: None,
                    }
                })
                .collect();
            // A row here is a record or a name, not a song, so the two orders
            // that read a song's own tags read them off the first track of the
            // group: the artist a record is by, and the record a name is on.
            let order = self.settings.order;
            let mut keyed: Vec<(SortKey, Row)> = self
                .rows
                .drain(..)
                .map(|row| {
                    let first = row.tracks.first().map(|i| &self.tracks[*i]);
                    (
                        SortKey {
                            name: row.label.to_lowercase(),
                            artist: first
                                .map(|t| t.album_artist.to_lowercase())
                                .unwrap_or_default(),
                            album: first.map(|t| t.album.to_lowercase()).unwrap_or_default(),
                            added: row
                                .tracks
                                .iter()
                                .map(|i| self.tracks[*i].added)
                                .max()
                                .unwrap_or(0),
                        },
                        row,
                    )
                })
                .collect();
            sort_rows(&mut keyed, order);
            self.rows = keyed.into_iter().map(|(_, row)| row).collect();
        } else {
            let mut indices = indices;
            // **The queue and an album keep an order of their own.** The queue
            // is the order it is going to play in, and an album is disc and
            // track number; a listing sorted over either of those is a listing
            // that is no longer about anything. Everywhere else the shelf is
            // in whatever order the Options menu was last set to.
            if self.tab != 4 && self.group.is_none() {
                let order = self.settings.order;
                let mut keyed: Vec<(SortKey, usize)> = indices
                    .drain(..)
                    .map(|index| {
                        let track = &self.tracks[index];
                        (
                            SortKey {
                                name: track.title.to_lowercase(),
                                artist: track.artist.to_lowercase(),
                                album: track.album.to_lowercase(),
                                added: track.added,
                            },
                            index,
                        )
                    })
                    .collect();
                sort_rows(&mut keyed, order);
                indices = keyed.into_iter().map(|(_, index)| index).collect();
            }
            self.rows = indices
                .into_iter()
                .enumerate()
                .map(|(at, index)| {
                    let track = &self.tracks[index];
                    Row {
                        label: track.title.clone(),
                        detail: track.artist.clone(),
                        cover: track.cover.clone(),
                        tracks: vec![index],
                        queue_at: (self.tab == 4).then_some(at),
                    }
                })
                .collect();
        }
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
    }

    pub fn change_tab(&mut self, tab: usize) {
        self.tab = tab;
        // The light on the rail follows the shelf, wherever the shelf was
        // chosen from — a bumper, a click, the menu. It only stops following
        // when it steps down onto one of the two buttons, which are not
        // shelves.
        self.rail = tab;
        self.group = None;
        self.selected = 0;
        self.arrived = self.seconds;
        self.rebuild();
    }

    /// What drawing found out, handed back at the end of the frame. Read by a
    /// press, which has to name a rectangle and cannot know one.
    pub fn placed(&mut self, placed: crate::draw::Placed) {
        self.cards = placed.cards;
        self.queue_cards = placed.queued;
        self.strip = placed.strip;
        self.volume_track = placed.volume;
        self.position_track = placed.position;
    }

    fn card_for(&self, index: usize) -> Option<Card> {
        self.cards.iter().copied().find(|card| card.index == index)
    }

    /// The rectangle a page opens out of when nothing better is known: the
    /// strip, which is always on the screen, and failing that the sleeve
    /// nothing has drawn yet.
    fn somewhere(&self) -> Card {
        if self.strip.rect[2] > 0.0 {
            self.strip
        } else {
            Card::nowhere()
        }
    }

    fn cross(&mut self, what: Change, from: Card, leaving: Vec<Row>) {
        self.crossing = Some(Crossing {
            what,
            back: false,
            from: from.rect,
            art: from.art,
            began: self.seconds,
            leaving,
            title: self.title(),
            subtitle: self.subtitle(),
        });
        self.arrived = self.seconds;
    }

    /// Open the Now Playing page out of a card.
    pub fn open_playing(&mut self, from: Card) {
        if self.screen == Screen::Playing || self.crossing.is_some() {
            return;
        }
        self.screen = Screen::Playing;
        self.zone = Zone::Transport;
        // That page has no record row of its own, so the light cannot stay on
        // the one it was pressed on.
        self.transport = self.transport.max(BAR_POSITION);
        self.queued = 0;
        self.light.clear();
        self.cross(Change::Playing, from, Vec::new());
    }

    fn leave_playing(&mut self) {
        if self.screen != Screen::Playing {
            return;
        }
        let from = self
            .card_for(self.selected)
            .or_else(|| self.cards.first().copied())
            .unwrap_or_else(|| self.somewhere());
        self.screen = Screen::Library;
        self.zone = Zone::Library;
        self.light.clear();
        self.crossing = Some(Crossing {
            what: Change::Playing,
            back: true,
            from: from.rect,
            art: from.art,
            began: self.seconds,
            leaving: Vec::new(),
            title: String::new(),
            subtitle: String::new(),
        });
        self.arrived = self.seconds;
    }

    fn leave_group(&mut self) {
        let Some(group) = self.group.take() else {
            return;
        };
        let leaving = std::mem::take(&mut self.rows);
        let title = group.title.clone();
        let subtitle = group.subtitle.clone();
        self.selected = 0;
        self.rebuild();
        // Back to the card the group was opened on, so that the list of songs
        // shrinks into the album it belongs to rather than into the corner.
        let index = self
            .rows
            .iter()
            .position(|row| row.label == title)
            .unwrap_or(0);
        self.selected = index;
        let from = self
            .card_for(index)
            .or_else(|| self.cards.first().copied())
            .unwrap_or_else(|| self.somewhere());
        self.crossing = Some(Crossing {
            what: Change::Group,
            back: true,
            from: from.rect,
            art: from.art,
            began: self.seconds,
            leaving,
            title,
            subtitle,
        });
        // Left where it was, for the reason `leave_playing` gives.
    }

    pub fn activate(&mut self) {
        let Some(row) = self.rows.get(self.selected).cloned() else {
            return;
        };
        let from = self
            .card_for(self.selected)
            .unwrap_or_else(|| self.somewhere());
        if self.group.is_none() && (self.tab == 1 || self.tab == 2) {
            let leaving = std::mem::take(&mut self.rows);
            let artist = if self.tab == 1 {
                row.detail.clone()
            } else {
                String::new()
            };
            self.group = Some(Group {
                title: row.label.clone(),
                subtitle: if self.tab == 1 {
                    message!("album-subtitle", "artist" => artist)
                } else {
                    i18n::text("artist-subtitle").into()
                },
                tracks: row.tracks.clone(),
            });
            self.selected = 0;
            self.rebuild();
            self.cross(Change::Group, from, leaving);
        } else if let Some(at) = row.queue_at {
            self.queue.select(at);
            self.start_current();
            self.open_playing(from);
        } else {
            let entries = self
                .rows
                .iter()
                .flat_map(|row| row.tracks.iter().copied())
                .collect();
            self.queue.replace(entries, self.selected);
            self.start_current();
            self.open_playing(from);
        }
    }

    pub fn start_current(&mut self) {
        let sleeve = self.current().and_then(|track| track.cover.clone());
        if sleeve != self.sleeve_was {
            self.sleeve_at = self.seconds;
            self.sleeve_was = sleeve;
        }
        if let Some(track) = self.current() {
            let path = track.path.clone();
            self.ended_handled = false;
            if !self.demo {
                self.audio.send(Command::Open(path));
            }
        } else {
            self.audio.send(Command::Stop);
        }
    }

    pub fn toggle(&mut self) {
        if self.queue.current().is_none() {
            if !self.queue.entries.is_empty() {
                self.queue.select(0);
                self.start_current();
            } else if self.group.is_none() && (self.tab == 1 || self.tab == 2) {
                if let Some(row) = self.rows.get(self.selected) {
                    self.queue.replace(row.tracks.clone(), 0);
                    self.start_current();
                }
            } else {
                self.activate();
            }
        } else if self.audio.status.ended || self.audio.status.error.is_some() {
            self.start_current();
        } else {
            self.audio.send(Command::Toggle);
        }
    }

    /// What pressing the thing the light is on does.
    ///
    /// The two bars answer a press as well as a direction, because a bar with
    /// nothing under South is a place on the row where the button a legend
    /// names does nothing. Both have an obvious one: the thing you are moving
    /// through is the thing you start and stop, and the thing you are setting
    /// the loudness of is the thing you silence.
    pub fn transport_action(&mut self, index: usize) {
        match index {
            NOW_PLAYING => {
                let from = self.strip;
                self.open_playing(from);
            }
            BAR_POSITION => self.toggle(),
            BAR_LOUDNESS => self.toggle_mute(),
            3 => {
                self.queue.shuffle = !self.queue.shuffle;
                self.queue.reset_shuffle();
                self.settings.shuffle = self.queue.shuffle;
                self.save();
            }
            4 => {
                self.queue.repeat = self.queue.repeat.cycle();
                self.settings.repeat = self.queue.repeat;
                self.save();
            }
            5 => {
                if self.audio.status.position > 3.0 {
                    self.audio.send(Command::Seek(0.0));
                } else if self.queue.previous().is_some() {
                    self.start_current();
                }
            }
            6 => self.toggle(),
            7 => {
                // Off the end of the queue there is nothing to move to, and
                // nothing is what Next does there rather than stopping what is
                // playing.
                let moved = self.queue.next(false).is_some();
                if moved {
                    self.start_current();
                }
            }
            _ => {}
        }
    }

    /// A direction along the strip: a step along the bar the light is on, or
    /// the button beside this one.
    ///
    /// **Left and Right belong to the bar while the light is on it.** Which is
    /// the whole of why Up and Down get on and off one: a bar you could walk
    /// off sideways is a bar you cannot set to anything but its two ends.
    pub fn nudge(&mut self, by: i32) {
        match self.transport {
            BAR_POSITION => self.seek_by(by as f64 * 10.0),
            BAR_LOUDNESS => self.hold_volume(self.settings.volume + by as f32 * 0.05),
            button => {
                let at = (button as i32 + by).clamp(FIRST_BUTTON as i32, TRANSPORT as i32 - 1);
                self.transport = at as usize;
            }
        }
    }

    /// The first row of the strip on the screen this is: the record that says
    /// what is playing, or — on the Now Playing page, which is what that
    /// record opens — the position bar.
    pub fn strip_top(&self) -> usize {
        if self.screen == Screen::Playing {
            BAR_POSITION
        } else {
            NOW_PLAYING
        }
    }

    /// Up and Down through the strip: the record, the position bar, the
    /// loudness bar, the buttons. Answers whether there was anywhere to go, so
    /// the screen above can take the light back when there is not.
    pub fn step_strip(&mut self, down: bool) -> bool {
        let top = self.strip_top();
        let to = match (self.transport, down) {
            (NOW_PLAYING, true) => BAR_POSITION,
            (BAR_POSITION, true) => BAR_LOUDNESS,
            (BAR_LOUDNESS, true) => FIRST_BUTTON,
            (BAR_POSITION, false) if top == BAR_POSITION => return false,
            (BAR_POSITION, false) => NOW_PLAYING,
            (BAR_LOUDNESS, false) => BAR_POSITION,
            (NOW_PLAYING, false) => return false,
            (_, false) => BAR_LOUDNESS,
            (_, true) => return false,
        };
        self.transport = to;
        true
    }

    /// Where the song is, as everything that draws it has to ask.
    ///
    /// Where the thumb has it while somebody is moving it, and where the sound
    /// really got to the rest of the time. See [`Drive`].
    pub fn position(&self) -> f64 {
        match &self.drive {
            Some(drive) if drive.what == BAR_POSITION => drive.to,
            _ => self.audio.status.position,
        }
    }

    /// How long the song is, where that is known at all.
    fn song_length(&self) -> Option<f64> {
        self.current()
            .map(|track| track.duration)
            .filter(|length| *length > 0.0)
    }

    /// Move through the song by so many seconds, stopping at either end.
    ///
    /// From where the thumb has it rather than from where the sound worker
    /// last said it was: five presses inside one report used to be one press,
    /// because each of them started again from the same stale number.
    pub fn seek_by(&mut self, seconds: f64) {
        let from = self.position();
        self.seek_at(from + seconds);
    }

    /// Move to a place in the song, as a share of its length. What a pointer
    /// on the groove says outright.
    pub fn seek_to(&mut self, part: f32) {
        let Some(length) = self.song_length() else {
            return;
        };
        self.seek_at(length * f64::from(part.clamp(0.0, 1.0)));
    }

    /// Take the song to a place in it, and hold it there.
    ///
    /// The worker hears at once the first time and on a cadence after that: a
    /// seek flushes a decoder, and a thumb leaning on the bar would otherwise
    /// ask for sixty of them a second, each racing the report of where the
    /// last one landed. See [`Drive`].
    fn seek_at(&mut self, at: f64) {
        let ceiling = self.song_length().unwrap_or(f64::MAX);
        let to = at.clamp(0.0, ceiling);
        let mut drive = match self.drive.take() {
            Some(drive) if drive.what == BAR_POSITION => drive,
            // The other bar, or nothing. Whatever was being held is finished
            // with before this one starts, so that letting go of one bar by
            // reaching for the other still writes down what the first did.
            was => {
                self.drive = was;
                self.let_go();
                Drive {
                    what: BAR_POSITION,
                    to,
                    told: SEEK_EVERY,
                    left: 0.0,
                }
            }
        };
        drive.to = to;
        drive.left = DRIVE_HELD;
        let now = drive.told >= SEEK_EVERY;
        if now {
            drive.told = 0.0;
        }
        self.drive = Some(drive);
        if now {
            self.audio.send(Command::Seek(to));
        }
    }

    /// What a push on the stick is worth, as a share of full speed.
    ///
    /// Squared past the dead zone, because a bar is set by eye: a thumb barely
    /// off centre is asking to creep, and a thumb at the stop is asking to
    /// cross the song. Straight, the useful half of the travel would be the
    /// first fifth of it and there would be no precision anywhere.
    fn drive_of(push: f32) -> f32 {
        let past = (push.abs() - STICK_DEAD) / (1.0 - STICK_DEAD);
        if past <= 0.0 {
            return 0.0;
        }
        (past * past).min(1.0).copysign(push)
    }

    /// Wind whichever bar the light is on along, at the speed the stick asks
    /// for.
    ///
    /// Nothing at all where the light is anywhere else: everywhere but on a
    /// bar the stick is the toolkit's own way of walking a list, and this must
    /// not become a second set of controls beside it.
    pub fn push_bar(&mut self, push: f32) {
        let rate = Self::drive_of(push);
        if rate == 0.0 || self.zone != Zone::Transport {
            return;
        }
        let dt = self.dt;
        match self.transport {
            BAR_POSITION => self.seek_by(f64::from(rate) * SEEK_RATE * f64::from(dt)),
            BAR_LOUDNESS => self.hold_volume(self.settings.volume + rate * LOUD_RATE * dt),
            _ => {}
        }
    }

    /// The drive is over: tell the worker where the thumb really left the
    /// song, and write down what a drive was deliberately not writing down
    /// sixty times a second.
    fn let_go(&mut self) {
        let Some(drive) = self.drive.take() else {
            return;
        };
        if drive.what != BAR_POSITION {
            self.save();
        } else if drive.told > 0.0 {
            // Only where something has happened since the last one. Letting go
            // of a bar nobody moved is not a seek.
            self.audio.send(Command::Seek(drive.to));
        }
    }

    /// Silence it, or give it back the loudness it had.
    ///
    /// The old loudness is remembered rather than jumped to a half, because a
    /// mute that came back at some other volume is not an undo. Silencing
    /// something already silent puts it back where it was, which is what a
    /// switch does.
    pub fn toggle_mute(&mut self) {
        if self.settings.volume > 0.0 {
            self.muted_from = self.settings.volume;
            self.set_volume(0.0);
        } else {
            let back = if self.muted_from > 0.0 {
                self.muted_from
            } else {
                0.5
            };
            self.set_volume(back);
        }
    }

    /// Set the loudness and write it down. One act, one setting remembered.
    pub fn set_volume(&mut self, level: f32) {
        self.hold_volume(level);
        self.let_go();
    }

    /// Set the loudness while somebody is still moving it.
    ///
    /// The sound follows at once and the settings file waits until they let
    /// go. Sixty writes a second is not a setting being remembered, it is a
    /// disk being worn out — and a pointer dragged along the groove, or a
    /// thumb leaning on the stick, asks for exactly that.
    fn hold_volume(&mut self, level: f32) {
        if self
            .drive
            .as_ref()
            .is_some_and(|drive| drive.what != BAR_LOUDNESS)
        {
            self.let_go();
        }
        self.settings.volume = level.clamp(0.0, 1.0);
        self.audio.send(Command::Volume(self.settings.volume));
        self.drive = Some(Drive {
            what: BAR_LOUDNESS,
            to: f64::from(self.settings.volume),
            told: 0.0,
            left: DRIVE_HELD,
        });
    }

    /// Whether the row under the light is one of the user's favourites.
    fn is_favourite(&self) -> bool {
        self.rows
            .get(self.selected)
            .map(|row| {
                !row.tracks.is_empty()
                    && row
                        .tracks
                        .iter()
                        .all(|i| self.settings.favourites.contains(&self.tracks[*i].path))
            })
            .unwrap_or(false)
    }

    /// What the Options menu offers, here and now.
    ///
    /// **No way to close the application.** Back does that, from the first
    /// page, as it does in the film player and the photo viewer beside this
    /// one; a menu row that quits is a row every other member of the family
    /// manages without, and one nobody looking for it in a menu would find
    /// before they found Back. It was here until 2026-09-19.
    fn open_menu(&mut self, page: &mut Page) {
        let (choices, marked) = self.menu_rows();
        page.menu_marked(Some(i18n::text("options")), &choices, marked);
    }

    /// The orders, raised from the row that names them.
    ///
    /// Out of the same anchor the Options menu came from — `light_at` was
    /// written down when the page was last drawn and nothing has moved the
    /// light since — so the orders unfold in the place Options was, which is
    /// what makes them read as what that row opened rather than as a second
    /// menu from somewhere else.
    fn open_orders(&mut self, page: &mut Page) {
        let (choices, marked) = self.order_rows();
        page.menu_marked(Some(i18n::text("sort-by")), &choices, marked);
    }

    /// What the menu says, and which of its rows wears the `chosen` mark.
    ///
    /// Split out from raising it so that what the menu offers can be asked
    /// without a window: every row of it is a decision about what this
    /// application can do from where it stands, and a decision is worth a
    /// test.
    fn menu_rows(&mut self) -> (Vec<&'static str>, usize) {
        let mut choices = Vec::new();
        self.menu.clear();
        // **Sorting is one row, and the orders are behind it.** They used to
        // be six rows at the head of this menu, which is how the film player
        // and the photo viewer do it — but those two draw their own menu, in
        // which a group of alternatives is separated by a rule and every row
        // carries the mark of what it is. This one is the toolkit's, which is
        // a plain list of names and one tick, so six bare nouns landed above
        // Play and Add to the queue with nothing to say they were a different
        // kind of thing at all. Reported as exactly that.
        //
        // Left out where it would name something that does nothing: inside an
        // album, which is disc and track number, and on the Queue shelf, which
        // is the order it is going to play in.
        if self.can_sort() {
            choices.push(i18n::text("sort-by"));
            self.menu.push(Menu::Sorting);
        }
        if self.screen == Screen::Library && !self.rows.is_empty() {
            choices.push(i18n::text("play"));
            self.menu.push(Menu::Play);
            choices.push(i18n::text("add-queue"));
            self.menu.push(Menu::Enqueue);
            if self.is_favourite() {
                choices.push(i18n::text("unfavourite"));
                self.menu.push(Menu::Unfavourite);
            } else {
                choices.push(i18n::text("favourite"));
                self.menu.push(Menu::Favourite);
            }
            if self.tab == 4 {
                choices.push(i18n::text("remove-queue"));
                self.menu.push(Menu::Remove);
            }
        }
        choices.extend([i18n::text("add-folder"), i18n::text("refresh")]);
        self.menu.extend([Menu::AddFolder, Menu::Refresh]);
        if !self.queue.entries.is_empty() {
            choices.push(i18n::text("clear-queue"));
            self.menu.push(Menu::Clear);
        }
        // Nothing in the Options menu is an answer to a question, so nothing
        // in it wears the `chosen` mark.
        (choices, usize::MAX)
    }

    /// Whether the shelf on screen has an order that is the user's to choose.
    ///
    /// Inside an album it is disc and track number, which is the record's own
    /// order and not a preference; on the Queue shelf it is the order the
    /// songs are going to play in.
    fn can_sort(&self) -> bool {
        self.screen == Screen::Library && self.group.is_none() && self.tab != 4
    }

    /// The orders, each its own row with the one in force ticked.
    ///
    /// A list rather than one row that cycles, for the reason the film player
    /// gives about its subtitle tracks: six orders behind one row would take
    /// six presses to find out what the sixth one was.
    fn order_rows(&mut self) -> (Vec<&'static str>, usize) {
        let mut choices = Vec::new();
        let mut marked = usize::MAX;
        self.menu.clear();
        for order in Order::ALL {
            if order == self.settings.order {
                marked = choices.len();
            }
            choices.push(order.label());
            self.menu.push(Menu::SortBy(order));
        }
        (choices, marked)
    }

    /// List the shelves in a different order, and remember it.
    ///
    /// **The row the light is on is kept across the change, not the place in
    /// the list.** A listing put in a different order is the same songs in a
    /// different order, and the light staying on the song somebody was looking
    /// at is what makes that visible — where a light that stayed on the fourth
    /// row would look like the listing had been replaced.
    pub fn sort_by(&mut self, order: Order) {
        let was = self
            .rows
            .get(self.selected)
            .and_then(|row| row.tracks.first().copied());
        self.settings.order = order;
        self.save();
        self.rebuild();
        if let Some(track) = was {
            if let Some(at) = self
                .rows
                .iter()
                .position(|row| row.tracks.first() == Some(&track))
            {
                self.selected = at;
            }
        }
    }

    fn menu_action(&mut self, action: Menu, page: &mut Page) {
        let tracks = self
            .rows
            .get(self.selected)
            .map(|row| row.tracks.clone())
            .unwrap_or_default();
        match action {
            Menu::Sorting => self.open_orders(page),
            Menu::Play => {
                self.queue.replace(tracks, 0);
                self.start_current();
            }
            Menu::Enqueue => {
                for index in tracks {
                    self.queue.append(index);
                }
                self.rebuild();
            }
            Menu::Favourite | Menu::Unfavourite => {
                let remove = matches!(action, Menu::Unfavourite);
                for index in tracks {
                    let path = self.tracks[index].path.clone();
                    if remove {
                        self.settings.favourites.remove(&path);
                    } else {
                        self.settings.favourites.insert(path);
                    }
                }
                self.save();
                self.rebuild();
            }
            Menu::Remove => {
                if self.queue.remove(self.selected) {
                    self.start_current();
                }
                self.rebuild();
            }
            Menu::AddFolder => {
                page.pick(PickerSelection::Folder, crate::settings::music_folder());
            }
            Menu::Refresh => {
                if !self.demo {
                    self.scan();
                }
            }
            Menu::Clear => {
                self.queue.replace(Vec::new(), 0);
                self.audio.send(Command::Stop);
                self.rebuild();
            }
            Menu::SortBy(order) => self.sort_by(order),
        }
    }

    /// Wind the crossing on, and let go of it once it has landed.
    ///
    /// Checked after the frame it lands on has been laid out, or the page is
    /// handed back on the very frame it was due to arrive and spends it flying
    /// off the card.
    fn advance(&mut self) {
        if self
            .crossing
            .as_ref()
            .is_some_and(|crossing| crossing.landed(self.seconds))
        {
            self.crossing = None;
        }
        // A stick and a pointer say every frame that they are still there; a
        // held direction arrives as one press and then as repeats of it. So a
        // drive is believed for a moment after the last of them and then let
        // go of, which is the same thing for all three.
        let dt = self.dt;
        let finished = match self.drive.as_mut() {
            Some(drive) => {
                drive.told += dt;
                drive.left -= dt;
                drive.left <= 0.0
            }
            None => false,
        };
        if finished {
            self.let_go();
        }
    }

    pub fn frame(&mut self, page: &mut Page) {
        // Everything that moves is a function of this and of when it began.
        // The clock starts at nought in a window and stands at whatever
        // `--shot` was given, which is why a settled picture is settled
        // without anything being told to settle.
        // How long this frame is, taken before the clock moves on. The only
        // thing here that needs it is a drive, which is the only thing here
        // that accumulates rather than being a function of `seconds`.
        self.dt = (page.seconds() - self.seconds).clamp(0.0, 0.25);
        self.seconds = page.seconds();
        self.frames += 1;

        // And the one thing a list of actions cannot carry: how far. Read
        // before the actions are, because a stick past the toolkit's own
        // threshold is *also* making Left and Right repeats out of the same
        // push, and on a bar this drives instead of them.
        self.pad.settle();
        let push = self.pad.push();

        let events: Vec<_> = self
            .receiver
            .as_ref()
            .map(|receiver| receiver.try_iter().collect())
            .unwrap_or_default();
        let changed = !events.is_empty();
        for event in events {
            self.scan_event(event);
        }
        if changed {
            self.rebuild();
        }
        self.audio.poll();
        if self.audio.status.ended && !self.ended_handled {
            self.ended_handled = true;
            if self.queue.next(true).is_some() {
                self.start_current();
            }
        }
        if self.sleeve_was.as_deref() != self.sleeve() {
            self.sleeve_at = self.seconds;
            self.sleeve_was = self.sleeve().map(|path| path.to_owned());
        }
        if let Some(path) = page.picked() {
            if !self.settings.folders.contains(&path) {
                self.settings.folders.push(path);
                self.save();
            }
            self.scan();
        }
        if let Some(index) = page.chose() {
            if let Some(action) = self.menu.get(index).copied() {
                self.menu_action(action, page);
            }
        }

        // A press made once the page has been laid out: `--shot` cannot press
        // anything before drawing has said where the cards went.
        if self.frames == 2 {
            if let Some(deferred) = self.deferred.take() {
                if let Some(action) = deferred.press {
                    self.act(action, page);
                }
                if let Some(crossing) = &mut self.crossing {
                    crossing.began = self.seconds - deferred.after;
                }
                self.arrived = self.seconds - deferred.after;
                // The dip is wound back with everything else, or `--after`
                // would mean one thing for the crossing and another for the
                // control that started it.
                self.pressed_at = self.seconds - deferred.after;
            }
        }

        // Where the light is on a bar and the stick is past the threshold the
        // toolkit makes repeats at, those repeats and this push are one hand.
        // Counting both would step the bar ten seconds on top of winding it
        // smoothly, which is the bar jumping out from under the thumb.
        let driving = self.zone == Zone::Transport
            && matches!(self.transport, BAR_POSITION | BAR_LOUDNESS)
            && push.abs() >= lxb_toolkit::input::STICK_ENGAGE;
        self.push_bar(push);

        let actions: Vec<_> = page.actions().collect();
        let scrolls: Vec<_> = page.scrolls().collect();
        for (index, action) in actions.into_iter().enumerate() {
            if driving && matches!(action, Action::Left | Action::Right) {
                continue;
            }
            if let Some(scroll) = scrolls.iter().find(|scroll| scroll.covers(index)) {
                if let lxb_render::Spot::Control(id) = scroll.spot {
                    if (crate::draw::ROW..crate::legend::SPOT).contains(&id) {
                        self.zone = Zone::Library;
                    }
                }
            }
            self.act(action, page);
        }
        for tab in 0..TABS.len() {
            if page.pressed(crate::draw::TAB + tab as u32) {
                self.taking_a_press(page);
                self.change_tab(tab);
                self.zone = Zone::Sidebar;
            }
        }
        for (slot, card) in self.cards.clone().into_iter().enumerate() {
            if page.pressed(crate::draw::ROW + slot as u32) {
                self.taking_a_press(page);
                self.selected = card.index;
                self.zone = Zone::Library;
                self.activate();
                break;
            }
        }
        for (slot, card) in self.queue_cards.clone().into_iter().enumerate() {
            if page.pressed(crate::draw::QUEUED + slot as u32) {
                self.taking_a_press(page);
                self.zone = Zone::Queue;
                self.queued = card.index;
                if card.index < self.queue.entries.len() {
                    self.queue.select(card.index);
                    self.start_current();
                }
                break;
            }
        }
        for index in FIRST_BUTTON..TRANSPORT {
            if page.pressed(crate::draw::CONTROL + index as u32) {
                self.taking_a_press(page);
                self.transport = index;
                self.zone = Zone::Transport;
                self.transport_action(index);
            }
        }
        if page.pressed(crate::draw::ADD_FOLDER) {
            self.taking_a_press(page);
            self.zone = Zone::Sidebar;
            self.rail = RAIL_ADD_FOLDER;
            page.pick(PickerSelection::Folder, crate::settings::music_folder());
        }
        // The one standing where the songs would be. It belongs to the
        // browser, so pressing it leaves the light there.
        if page.pressed(crate::draw::ADD_FOLDER_HERE) {
            self.taking_a_press(page);
            self.zone = Zone::Library;
            page.pick(PickerSelection::Folder, crate::settings::music_folder());
        }
        if page.pressed(crate::draw::OPTIONS) {
            self.taking_a_press(page);
            self.zone = Zone::Sidebar;
            self.rail = RAIL_OPTIONS;
            self.open_menu(page);
        }
        if page.pressed(crate::draw::HEADING) {
            self.taking_a_press(page);
            self.leave_group();
        }
        // A bar is set by where the pointer is, and goes on being set while
        // the button is held down: a groove you can only click is a groove you
        // cannot aim. The sound and the dip belong to the press that began it,
        // not to every frame of the drag that followed.
        let grabbed = page.pressed(crate::draw::VOLUME);
        if let Some(level) = Self::along(page, crate::draw::VOLUME, self.volume_track) {
            if grabbed {
                self.taking_a_press(page);
            }
            self.zone = Zone::Transport;
            self.transport = BAR_LOUDNESS;
            self.hold_volume(level);
        }
        let grabbed = page.pressed(crate::draw::POSITION);
        if let Some(part) = Self::along(page, crate::draw::POSITION, self.position_track) {
            if grabbed {
                self.taking_a_press(page);
            }
            self.zone = Zone::Transport;
            self.transport = BAR_POSITION;
            self.seek_to(part);
        }
        if page.pressed(crate::draw::STRIP) {
            self.taking_a_press(page);
            self.zone = Zone::Transport;
            self.transport = NOW_PLAYING;
            let from = self.strip;
            self.open_playing(from);
        }
        let hints = crate::draw::hints(self);
        for i in 0..hints.len() {
            let id = crate::legend::SPOT + i as u32;
            if page.pressed(id) {
                if let Some(button) = crate::legend::pressed(id, &hints, page.pad_in_hand()) {
                    self.act(button.action(), page);
                }
            }
        }
        crate::draw::draw(self, page);
        self.advance();
    }

    /// How far along a groove the pointer has this bar, 0 at its foot and 1 at
    /// its head — for as long as the button that landed on it is held down.
    ///
    /// *Where* the pointer is is the whole of the answer, which is the one
    /// thing a spot cannot say on its own — so the groove writes its own
    /// rectangle down and the pointer is read against it.
    ///
    /// **[`Page::dragging`] is that pointer.** It is also the only thing in
    /// the toolkit that says where the pointer *is*: `Page::cursor` is the
    /// layout's own cursor — where the next thing would be laid out — and
    /// reading it for a pointer is why a click anywhere on either bar used to
    /// set it to nought. It answers every frame from the press until the
    /// button comes up, which is the same thing as a drag and is why there is
    /// nothing else here to make one.
    ///
    /// The whole row is the target and the groove is the ruler: a pointer on
    /// the row but short of the groove's own left-hand end is at the foot of
    /// it, which is what clamping says. Aiming at a bar and missing it by the
    /// height of a clock is not a press that should be thrown away.
    fn along(page: &mut Page, spot: u32, track: [f32; 4]) -> Option<f32> {
        Self::along_track(page.dragging(spot)?[0], track)
    }

    /// The arithmetic of that, with no pointer in it, so that it can be asked
    /// a question a window has to be open to ask.
    fn along_track(x: f32, track: [f32; 4]) -> Option<f32> {
        let [from, _, across, _] = track;
        if across <= 0.0 {
            return None;
        }
        Some(((x - from) / across).clamp(0.0, 1.0))
    }

    /// A pointer press is a press: the same sound and the same dip a button
    /// makes under a pad.
    fn taking_a_press(&mut self, page: &mut Page) {
        self.pressed_at = self.seconds;
        page.play(Sound::Press);
    }

    fn act(&mut self, action: Action, page: &mut Page) {
        // A crossing takes no presses. Both pages have written their targets
        // down, and a press landing in the middle of one is a press on
        // whichever page happened to be drawn second.
        if self.crossing.is_some() {
            return;
        }
        match action {
            Action::Menu => {
                page.play(Sound::Press);
                self.open_menu(page);
            }
            Action::Submit => {
                self.pressed_at = self.seconds;
                page.play(Sound::Press);
                self.toggle();
            }
            Action::Next if self.screen == Screen::Library => {
                page.play(Sound::Move);
                self.change_tab((self.tab + 1) % TABS.len());
            }
            Action::Previous if self.screen == Screen::Library => {
                page.play(Sound::Move);
                self.change_tab((self.tab + TABS.len() - 1) % TABS.len());
            }
            // **Back is the way out, and the last Back closes Music.** Off
            // the Now Playing page to the library, out of an album to the wall
            // it was on, off the browser to the rail, and off the rail out of
            // the application — the same last step the film player and the
            // photo viewer take, and the one the legend has always named: the
            // corner says Close there and used to open the Options menu, which
            // is a legend naming a button that does something else.
            Action::Back => {
                page.play(Sound::Back);
                if self.screen == Screen::Playing {
                    self.leave_playing();
                } else if self.group.is_some() {
                    self.leave_group();
                } else if self.zone != Zone::Sidebar {
                    self.zone = Zone::Sidebar;
                } else {
                    page.quit();
                }
            }
            Action::Accept => {
                self.pressed_at = self.seconds;
                page.play(Sound::Press);
                match (self.screen, self.zone) {
                    (Screen::Library, Zone::Library) => {
                        if self.rows.is_empty() {
                            page.pick(PickerSelection::Folder, crate::settings::music_folder());
                        } else {
                            self.activate();
                        }
                    }
                    // A shelf opens the browser on it; the two buttons at the
                    // foot of the rail do what they say.
                    (Screen::Library, Zone::Sidebar) => match self.rail {
                        RAIL_ADD_FOLDER => {
                            page.pick(PickerSelection::Folder, crate::settings::music_folder());
                        }
                        RAIL_OPTIONS => self.open_menu(page),
                        _ => self.zone = Zone::Library,
                    },
                    (Screen::Playing, Zone::Queue) => {
                        let at = self.queued;
                        if at < self.queue.entries.len() {
                            self.queue.select(at);
                            self.start_current();
                        }
                    }
                    _ => self.transport_action(self.transport),
                }
            }
            direction => {
                page.play(Sound::Move);
                self.navigate(direction);
            }
        }
    }

    /// Hand the light to the strip, and remember what handed it.
    ///
    /// **Always at the top of the strip**, whichever zone it came from and
    /// wherever the light was in the strip last. A direction means the thing
    /// next to this one; a strip that gave the light back to whatever row it
    /// was on the last time it was there is a Down that lands somewhere
    /// different every time it is pressed, which is what "the controls seem
    /// completely random" was.
    fn enter_strip(&mut self, from: Zone) {
        self.strip_from = from;
        self.zone = Zone::Transport;
        self.transport = self.strip_top();
    }

    /// Where a direction takes the light on the library screen.
    ///
    /// **One model, and it is the screen.** The rail and the browser stand
    /// side by side with the strip across the foot of both:
    ///
    /// ```text
    ///   rail │ browser
    ///  ──────┴─────────
    ///        strip
    /// ```
    ///
    /// Up and Down walk a zone's rows, Left and Right its columns, and falling
    /// off an edge hands the light to the zone that is really there — never to
    /// one that happens to be written down. What it lands on is the row of
    /// that zone nearest where it came from: the top of the strip coming down,
    /// and whatever handed the light over going back up.
    fn navigate(&mut self, direction: Action) {
        if self.screen == Screen::Playing {
            return self.navigate_playing(direction);
        }
        match self.zone {
            // The five shelves, then Add a folder, then Options. Off the foot
            // of it is the strip; there is nothing to the left of it.
            Zone::Sidebar => match direction {
                Action::Up => {
                    if self.rail > 0 {
                        self.stand_on_rail(self.rail - 1);
                    }
                }
                Action::Down => {
                    if self.rail + 1 < RAIL_ROWS {
                        self.stand_on_rail(self.rail + 1);
                    } else {
                        self.enter_strip(Zone::Sidebar);
                    }
                }
                Action::Right => self.zone = Zone::Library,
                _ => {}
            },
            // A list is one column and a wall is several. Off the foot of
            // either is the strip; off the left-hand edge is the rail; there
            // is nothing to the right of the last column.
            Zone::Library => {
                let step = if self.grid() { self.columns.max(1) } else { 1 };
                match direction {
                    Action::Up => self.selected = self.selected.saturating_sub(step),
                    Action::Down => {
                        if self.selected + step < self.rows.len() {
                            self.selected += step;
                        } else {
                            self.enter_strip(Zone::Library);
                        }
                    }
                    Action::Left => {
                        if self.grid() && !self.selected.is_multiple_of(step) {
                            self.selected -= 1;
                        } else {
                            self.zone = Zone::Sidebar;
                        }
                    }
                    // There is nothing to the right of the last column: the
                    // strip is under the browser, not beside it.
                    Action::Right
                        if self.grid()
                            && self.selected % step + 1 < step
                            && self.selected + 1 < self.rows.len() =>
                    {
                        self.selected += 1
                    }
                    _ => {}
                }
            }
            // The record, the two bars, the buttons. Left and Right belong to
            // whichever of those the light is on; Up off the top of it gives
            // the light back to whatever handed it over.
            _ => match direction {
                Action::Up => {
                    if !self.step_strip(false) {
                        self.zone = self.strip_from;
                    }
                }
                Action::Down => {
                    self.step_strip(true);
                }
                Action::Left => self.nudge(-1),
                Action::Right => self.nudge(1),
                _ => {}
            },
        }
    }

    /// Put the light on a row of the rail, and open that shelf if the row is
    /// one.
    fn stand_on_rail(&mut self, row: usize) {
        if row < TABS.len() {
            self.change_tab(row);
        } else {
            self.rail = row.min(RAIL_ROWS - 1);
        }
    }

    /// The same model on the Now Playing page, which is laid out the other way
    /// round:
    ///
    /// ```text
    ///   record and words │ up next
    ///  ──────────────────┴─────────
    ///          strip
    /// ```
    ///
    /// The queue stands above the strip rather than beside it, so Down off the
    /// end of the queue is the strip and Up off the top of the strip is the
    /// queue — the same two directions that walk between the browser and the
    /// strip on the other screen. Left out of the queue is the strip as well,
    /// because a queue can run to hundreds and pressing Down through all of
    /// them to reach Play is not a way out.
    fn navigate_playing(&mut self, direction: Action) {
        let queue = self.queue.entries.len();
        match self.zone {
            Zone::Queue => match direction {
                Action::Up => self.queued = self.queued.saturating_sub(1),
                Action::Down => {
                    if self.queued + 1 < queue {
                        self.queued += 1;
                    } else {
                        self.enter_strip(Zone::Queue);
                    }
                }
                Action::Left => self.enter_strip(Zone::Queue),
                _ => {}
            },
            _ => {
                self.zone = Zone::Transport;
                match direction {
                    Action::Up => {
                        if !self.step_strip(false) && queue > 0 {
                            // The row of the queue nearest the strip, which is
                            // the last of them.
                            self.zone = Zone::Queue;
                            self.queued = queue - 1;
                        }
                    }
                    Action::Down => {
                        self.step_strip(true);
                    }
                    Action::Left => self.nudge(-1),
                    Action::Right => self.nudge(1),
                    _ => {}
                }
            }
        }
    }

    fn demo_library(&mut self) {
        for (artist, album, titles) in [
            (
                "Juniper Coast",
                "The quiet hours",
                [
                    "First light",
                    "Open windows",
                    "A little further",
                    "Sunday, slowly",
                ],
            ),
            (
                "Mira Sol",
                "After the rain",
                ["Soft focus", "Blue hour", "Somewhere warm", "Home again"],
            ),
            (
                "Northbound",
                "Outside / Inside",
                ["Night drive", "Signals", "Long way home", "Stay awhile"],
            ),
        ] {
            for (n, title) in titles.iter().enumerate() {
                self.tracks.push(Track {
                    path: PathBuf::from(format!("/preview/{artist}/{title}.flac")),
                    title: (*title).into(),
                    artist: artist.into(),
                    album_artist: artist.into(),
                    album: album.into(),
                    disc: 1,
                    number: n as u32 + 1,
                    duration: 183.0 + n as f64 * 27.0,
                    cover: None,
                    // A made-up library needs a made-up clock, so that the
                    // two orders that read one have something to read.
                    added: 1_700_000_000 + self.tracks.len() as u64 * 3_600,
                });
            }
        }
        self.queue.replace((0..self.tracks.len()).collect(), 0);
        self.settings.favourites.insert(self.tracks[0].path.clone());
        self.rebuild();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fault this is here for: the pointer was read from `Page::cursor`,
    /// which is the *layout's* cursor — where the next thing would be laid out
    /// — and not the pointer at all. It stands at the left-hand edge of the
    /// page, so every press anywhere on either bar answered the same number
    /// and this clamped it to nought: a click on the position bar sent the
    /// song back to its beginning, and a click on the loudness bar silenced
    /// it. Where the pointer really is now comes from `Page::dragging`, which
    /// is the only thing in the toolkit that says.
    #[test]
    fn a_point_along_a_groove_is_where_it_is_along_it() {
        let track = [200.0, 50.0, 400.0, 4.0];
        assert_eq!(Music::along_track(200.0, track), Some(0.0));
        assert_eq!(Music::along_track(400.0, track), Some(0.5));
        assert_eq!(Music::along_track(600.0, track), Some(1.0));
        // The whole row is the target and the groove is only the ruler, so
        // short of it and past it are its two ends rather than nothing.
        assert_eq!(Music::along_track(0.0, track), Some(0.0));
        assert_eq!(Music::along_track(9_000.0, track), Some(1.0));
        // And a row too narrow to have been given a groove answers nothing.
        assert_eq!(Music::along_track(400.0, [200.0, 50.0, 0.0, 4.0]), None);
    }

    /// Further is faster, and the near half of the travel is the slow half.
    #[test]
    fn the_stick_drives_a_bar_by_how_far_it_is_pushed() {
        assert_eq!(Music::drive_of(0.0), 0.0);
        assert_eq!(
            Music::drive_of(STICK_DEAD),
            0.0,
            "a resting thumb is not a push"
        );
        assert_eq!(Music::drive_of(1.0), 1.0, "and the stop is full speed");
        assert_eq!(Music::drive_of(-1.0), -1.0, "either way about");

        let half = Music::drive_of(0.5);
        let full = Music::drive_of(1.0);
        assert!(
            half > 0.0 && half < full * 0.25,
            "half way over is a creep, not half speed: {half}"
        );
        // Monotonic the whole way, or there is a place on the stick where
        // pushing further slows down.
        let mut last = 0.0;
        for step in 0..=100 {
            let now = Music::drive_of(step as f32 / 100.0);
            assert!(now >= last, "the stick went backwards at {step}");
            last = now;
        }
    }

    /// The bar the light is on is the bar the stick moves, and no other.
    #[test]
    fn a_push_moves_the_bar_under_the_light_and_nothing_else() {
        let mut music = Music::new(None, true, true);
        music.dt = 1.0 / 60.0;
        music.zone = Zone::Transport;

        music.transport = BAR_LOUDNESS;
        music.set_volume(0.5);
        music.push_bar(1.0);
        let louder = music.settings.volume;
        assert!(louder > 0.5, "a push at the stop did not move the loudness");
        assert!(
            louder - 0.5 < LOUD_RATE * music.dt * 1.01,
            "and it moved further in one frame than one frame is worth"
        );

        // On a button, the same push is the toolkit's own way of walking the
        // row and this must keep out of it.
        music.transport = FIRST_BUTTON;
        let was = music.settings.volume;
        music.push_bar(-1.0);
        assert_eq!(music.settings.volume, was);

        // And on the shelf, where the stick is how the list is walked at all.
        music.zone = Zone::Library;
        music.transport = BAR_LOUDNESS;
        music.push_bar(-1.0);
        assert_eq!(music.settings.volume, was);
    }

    /// A drive is the thumb's own idea of where the song is, so that a groove
    /// does not snap back to the worker's last report between seeks — and so
    /// that presses inside one report accumulate instead of all starting from
    /// the same stale number.
    #[test]
    fn a_drive_is_what_the_song_shows_until_it_is_let_go_of() {
        let mut music = Music::new(None, true, true);
        music.dt = 1.0 / 60.0;
        let length = music.current().map(|track| track.duration).unwrap_or(0.0);
        assert!(length > 60.0, "the made-up library has songs to seek in");

        assert_eq!(music.position(), music.audio.status.position);
        music.seek_to(0.5);
        assert!((music.position() - length * 0.5).abs() < 0.001);
        // Nothing has been reported back, and the bar stays where it was put.
        music.seek_by(10.0);
        assert!((music.position() - (length * 0.5 + 10.0)).abs() < 0.001);
        // Either end holds.
        music.seek_by(-1.0e6);
        assert_eq!(music.position(), 0.0);
        music.seek_by(1.0e6);
        assert!((music.position() - length).abs() < 0.001);

        // Held for a moment after the last of it, and then let go of.
        for _ in 0..(DRIVE_HELD / music.dt) as usize + 2 {
            music.advance();
        }
        assert!(
            music.drive.is_none(),
            "a drive nobody is holding was not let go of"
        );
        assert_eq!(music.position(), music.audio.status.position);
    }

    /// Reaching for one bar finishes with the other, or the loudness a thumb
    /// left behind would never reach the settings file.
    #[test]
    fn moving_the_other_bar_lets_go_of_the_first() {
        let mut music = Music::new(None, true, true);
        music.dt = 1.0 / 60.0;
        music.hold_volume(0.4);
        assert!(music.drive.is_some());
        music.seek_to(0.25);
        assert_eq!(
            music.drive.as_ref().map(|drive| drive.what),
            Some(BAR_POSITION),
            "the song is what is being moved now"
        );
        assert_eq!(
            music.settings.volume, 0.4,
            "and the loudness stayed where it was put"
        );
    }

    #[test]
    fn browsing_does_not_replace_the_playing_queue() {
        let mut music = Music::new(None, true, true);
        let original = music.queue.entries.clone();
        music.change_tab(1);
        music.activate();
        assert!(music.group.is_some());
        assert_eq!(music.queue.entries, original);
        assert_eq!(music.queue.current(), Some(0));
        music.crossing = None;
        music.selected = 1;
        music.activate();
        assert_eq!(music.queue.entries.len(), 4);
        assert_eq!(music.queue.position, Some(1));
    }

    /// Down out of the browser lands on the strip, Right walks to the last
    /// button and stops there, and Up climbs back out of the strip one row at
    /// a time before it lets go of it.
    #[test]
    fn every_transport_control_is_reachable_and_navigation_is_bounded() {
        let mut music = Music::new(None, true, true);
        // Down out of the browser lands on the top row of the strip, wherever
        // the light was in the strip last.
        music.zone = Zone::Transport;
        music.transport = TRANSPORT - 1;
        music.zone = Zone::Library;
        for _ in 0..100 {
            if music.zone == Zone::Transport {
                break;
            }
            music.navigate(Action::Down);
        }
        assert_eq!(
            music.transport, NOW_PLAYING,
            "Down into the strip has to land on the same row every time"
        );
        for _ in 0..20 {
            music.navigate(Action::Down);
        }
        assert_eq!(
            music.transport, FIRST_BUTTON,
            "and Down stops at the row of buttons"
        );
        for _ in 0..20 {
            music.navigate(Action::Right);
        }
        assert_eq!(music.transport, TRANSPORT - 1);
        // Three rows to climb before the browser hands the light back.
        for expected in [BAR_LOUDNESS, BAR_POSITION, NOW_PLAYING] {
            music.navigate(Action::Up);
            assert_eq!(music.transport, expected);
            assert!(music.zone == Zone::Transport);
        }
        music.navigate(Action::Up);
        assert!(
            music.zone == Zone::Library,
            "Up off the strip undoes the Down that got there"
        );
        music.navigate(Action::Left);
        assert!(music.zone == Zone::Sidebar);
        for _ in 0..20 {
            music.navigate(Action::Up);
        }
        assert_eq!(music.rail, 0);
        assert_eq!(music.tab, 0);
        music.navigate(Action::Right);
        assert!(music.zone == Zone::Library);
    }

    /// The rail is seven rows, not five: the two buttons at the foot of it are
    /// walked to and pressed like everything else.
    ///
    /// They were numbered for a pointer and nothing else, so on a machine with
    /// a controller and no mouse neither of them could be reached at all.
    #[test]
    fn the_two_buttons_at_the_foot_of_the_rail_are_rows_of_it() {
        let mut music = Music::new(None, true, true);
        music.zone = Zone::Sidebar;
        for row in 1..RAIL_ROWS {
            music.navigate(Action::Down);
            assert_eq!(music.rail, row);
            assert!(music.zone == Zone::Sidebar, "row {row} fell off the rail");
        }
        assert_eq!(music.rail, RAIL_OPTIONS);
        // Standing on a button does not change which shelf is open.
        assert_eq!(music.tab, TABS.len() - 1);

        // And off the foot of the rail is the strip, at the top of it — the
        // same row the browser hands it to.
        music.navigate(Action::Down);
        assert!(music.zone == Zone::Transport);
        assert_eq!(music.transport, NOW_PLAYING);
        music.navigate(Action::Up);
        assert!(
            music.zone == Zone::Sidebar,
            "Up off the strip went somewhere the light did not come from"
        );
        assert_eq!(music.rail, RAIL_OPTIONS);

        // Back up the rail, and the shelf follows the light again the moment
        // it is on one.
        music.navigate(Action::Up);
        assert_eq!(music.rail, RAIL_ADD_FOLDER);
        assert_eq!(music.tab, TABS.len() - 1);
        music.navigate(Action::Up);
        assert_eq!(music.rail, TABS.len() - 1);
        assert_eq!(music.tab, TABS.len() - 1);
    }

    /// A direction along a bar moves the bar and nothing else: the light does
    /// not walk off it sideways, which is the whole reason Up and Down are
    /// what get on and off one.
    #[test]
    fn a_direction_along_a_bar_moves_the_bar_and_not_the_light() {
        let mut music = Music::new(None, true, true);
        music.zone = Zone::Transport;
        music.transport = BAR_LOUDNESS;
        music.settings.volume = 0.5;
        for _ in 0..4 {
            music.navigate(Action::Right);
        }
        assert_eq!(music.transport, BAR_LOUDNESS, "the light stayed on the bar");
        assert!(music.settings.volume > 0.69 && music.settings.volume < 0.71);
        for _ in 0..40 {
            music.navigate(Action::Left);
        }
        assert_eq!(music.transport, BAR_LOUDNESS);
        assert_eq!(music.settings.volume, 0.0, "and it stops at the end");
    }

    /// Silencing it and turning it back on is a switch, and it comes back
    /// where it was rather than at some number nobody chose.
    #[test]
    fn muting_remembers_the_loudness_it_silenced() {
        let mut music = Music::new(None, true, true);
        music.set_volume(0.8);
        music.toggle_mute();
        assert_eq!(music.settings.volume, 0.0);
        music.toggle_mute();
        assert!((music.settings.volume - 0.8).abs() < 1e-6);
    }

    #[test]
    fn same_album_title_from_different_artists_stays_separate() {
        let mut music = Music::new(None, true, true);
        music.tracks[4].album = music.tracks[0].album.clone();
        music.change_tab(1);
        assert_eq!(
            music
                .rows
                .iter()
                .filter(|row| row.label == "The quiet hours")
                .count(),
            2
        );
    }

    #[test]
    fn favourites_filter_and_duplicate_queue_occurrences_are_preserved() {
        let mut music = Music::new(None, true, true);
        music.change_tab(3);
        assert_eq!(music.rows.len(), 1);
        music.queue.replace(vec![3, 3, 5], 0);
        music.change_tab(4);
        music.selected = 1;
        music.activate();
        assert_eq!(music.queue.position, Some(1));
        assert_eq!(music.rows.len(), 3);
    }

    /// A crossing is one number from the card to the page and back again, and
    /// the page standing over the other one is solid long before the rectangle
    /// has arrived.
    #[test]
    fn a_crossing_starts_on_the_card_and_ends_on_the_page() {
        let mut music = Music::new(None, true, true);
        music.seconds = 4.0;
        music.open_playing(Card {
            index: 0,
            rect: [10.0, 20.0, 100.0, 100.0],
            art: [12.0, 22.0, 96.0, 96.0],
        });
        let crossing = music.crossing.as_ref().expect("a crossing");
        assert_eq!(crossing.grown(4.0), 0.0);
        assert_eq!(crossing.grown(4.0 + duration::LAUNCH_OPEN), 1.0);
        assert!(crossing.grown(4.0 + duration::LAUNCH_OPEN * 0.5) > 0.0);
        // Solid well before the rectangle has arrived: half way along the
        // crossing the page standing over the other one is already whole.
        assert!(crossing.arriving(4.0 + duration::LAUNCH_OPEN * 0.5) >= 1.0);
        assert!(crossing.arriving(4.0) < 0.001);
        assert!(!crossing.landed(4.0));
        assert!(crossing.landed(4.0 + duration::LAUNCH_OPEN));
    }

    /// And the way back is the same number read backwards, out of the card the
    /// page is going into rather than the one it came out of.
    #[test]
    fn going_back_unwinds_the_same_crossing() {
        let mut music = Music::new(None, true, true);
        music.seconds = 1.0;
        music.placed(crate::draw::Placed {
            cards: vec![Card {
                index: 0,
                rect: [4.0, 5.0, 60.0, 60.0],
                art: [4.0, 5.0, 60.0, 60.0],
            }],
            ..Default::default()
        });
        music.open_playing(music.cards[0]);
        music.crossing = None;
        music.seconds = 2.0;
        music.leave_playing();
        let crossing = music.crossing.as_ref().expect("a crossing back");
        assert!(crossing.back);
        assert_eq!(crossing.from, [4.0, 5.0, 60.0, 60.0]);
        assert_eq!(crossing.grown(2.0), 1.0);
        assert!(crossing.grown(2.0 + duration::LAUNCH_OPEN) < 1e-6);
        // A frame never falls exactly on the end of a duration, and the
        // subtraction that works out how far along it is loses the last bit of
        // a float anyway. What matters is that the frame after it has landed.
        assert!(!crossing.landed(2.0));
        assert!(crossing.landed(2.0 + duration::LAUNCH_OPEN + 1.0 / 60.0));
        assert!(matches!(music.screen, Screen::Library));
    }

    /// Leaving an album puts the light back on the album it was opened from,
    /// so the songs shrink into their own card and not into the corner.
    #[test]
    fn leaving_an_album_goes_back_into_its_own_card() {
        let mut music = Music::new(None, true, true);
        music.change_tab(1);
        let wanted = music.rows[1].label.clone();
        music.selected = 1;
        music.placed(crate::draw::Placed {
            cards: vec![
                Card {
                    index: 0,
                    rect: [0.0, 0.0, 10.0, 10.0],
                    art: [0.0, 0.0, 10.0, 10.0],
                },
                Card {
                    index: 1,
                    rect: [20.0, 0.0, 10.0, 10.0],
                    art: [20.0, 0.0, 10.0, 10.0],
                },
            ],
            ..Default::default()
        });
        music.activate();
        music.crossing = None;
        assert!(music.group.is_some());
        let songs = music.rows.len();
        music.leave_group();
        let crossing = music.crossing.as_ref().expect("a crossing back");
        assert_eq!(crossing.leaving.len(), songs);
        assert_eq!(music.rows[music.selected].label, wanted);
        assert_eq!(crossing.from, [20.0, 0.0, 10.0, 10.0]);
    }

    /// A press dips the control under the light and lets it back up, and a
    /// session in which nothing has been pressed dips nothing at all.
    #[test]
    fn a_press_dips_the_control_under_the_light_and_then_stops() {
        let mut music = Music::new(None, true, true);
        assert_eq!(music.press_through(), None, "nothing has been pressed yet");
        music.seconds = 5.0;
        music.pressed_at = 5.0;
        assert_eq!(music.press_through(), Some(0.0));
        music.seconds = 5.0 + duration::GUIDE_PRESS * 0.5;
        assert!(music
            .press_through()
            .is_some_and(|t| (0.4..0.6).contains(&t)));
        // And the curve it is read through takes the control down and brings it
        // all the way back to its own size.
        assert_eq!(motion::press_scale(0.0), 1.0);
        assert!(motion::press_scale(motion::PRESS_DOWN) < 1.0);
        music.seconds = 5.0 + duration::GUIDE_PRESS * 1.01;
        assert_eq!(music.press_through(), None, "and then it is over");
    }

    /// The rows of a page come in one after another, and a page that arrived
    /// long ago is entirely still.
    #[test]
    fn rows_arrive_one_after_another_and_then_stop_moving() {
        assert_eq!(entry(0.0, 0), 0.0);
        assert!(entry(duration::ENTRY_LEAD + duration::ENTRY_SLIDE * 0.5, 0) > 0.0);
        assert!(entry(0.2, 0) > entry(0.2, 6));
        assert_eq!(entry(10.0, 40), 1.0);
    }

    /// Right off the end of the transport is the queue, and Left is the way
    /// back — on a page with nothing queued there is nowhere to the right.
    #[test]
    fn the_now_playing_page_has_two_places_to_be() {
        let mut music = Music::new(None, true, true);
        music.screen = Screen::Playing;
        music.zone = Zone::Transport;
        music.transport = BAR_POSITION;
        // The queue stands above the strip, so Up off the top of the strip is
        // the queue and Left out of the queue is the strip again.
        music.navigate(Action::Up);
        assert!(music.zone == Zone::Queue);
        assert_eq!(
            music.queued,
            music.queue.entries.len() - 1,
            "the row of the queue nearest the strip"
        );
        music.navigate(Action::Left);
        assert!(music.zone == Zone::Transport);
        assert_eq!(music.transport, BAR_POSITION);
        // Down off the end of the queue is the strip as well.
        music.zone = Zone::Queue;
        music.queued = music.queue.entries.len() - 1;
        music.navigate(Action::Down);
        assert!(music.zone == Zone::Transport);
        assert_eq!(music.transport, BAR_POSITION);
        // With nothing queued there is nowhere above the strip to go.
        music.queue.replace(Vec::new(), 0);
        music.zone = Zone::Transport;
        music.transport = BAR_POSITION;
        music.navigate(Action::Up);
        assert!(music.zone == Zone::Transport);
    }

    /// On the Now Playing page the strip is the same three rows walked the
    /// same way, and Right along a bar is the bar moving and never the light
    /// leaving it.
    #[test]
    fn the_now_playing_strip_walks_the_same_way_and_still_reaches_the_queue() {
        let mut music = Music::new(None, true, true);
        music.selected = 0;
        music.activate();
        music.crossing = None;
        assert!(music.screen == Screen::Playing);
        music.zone = Zone::Transport;

        music.transport = BAR_POSITION;
        music.navigate(Action::Down);
        assert_eq!(music.transport, BAR_LOUDNESS);
        music.navigate(Action::Down);
        assert_eq!(music.transport, FIRST_BUTTON);
        music.navigate(Action::Up);
        assert_eq!(music.transport, BAR_LOUDNESS);

        // Right along a bar is the bar moving, never the light leaving it.
        music.navigate(Action::Right);
        assert!(music.zone == Zone::Transport, "a bar handed the light away");
        assert_eq!(music.transport, BAR_LOUDNESS);

        // And Right off the end of the buttons goes nowhere: what is beside
        // the last of them is the edge of the page.
        music.transport = TRANSPORT - 1;
        music.navigate(Action::Right);
        assert!(music.zone == Zone::Transport);
        assert_eq!(music.transport, TRANSPORT - 1);
    }

    /// The record at the head of the strip is a place the light can stand, and
    /// pressing it opens the Now Playing page — which is the whole of the way
    /// back to what is playing on a machine with no mouse on it.
    #[test]
    fn the_record_at_the_head_of_the_strip_opens_now_playing() {
        let mut music = Music::new(None, true, true);
        music.zone = Zone::Transport;
        music.transport = FIRST_BUTTON;
        for expected in [BAR_LOUDNESS, BAR_POSITION, NOW_PLAYING] {
            music.navigate(Action::Up);
            assert_eq!(music.transport, expected);
        }
        assert!(music.screen == Screen::Library);
        music.transport_action(NOW_PLAYING);
        assert!(music.screen == Screen::Playing);
        // And that page has no such row, so the light comes to rest on the
        // first thing it does have.
        assert!(music.transport >= BAR_POSITION);
        music.crossing = None;
        music.navigate(Action::Up);
        assert_eq!(
            music.transport, BAR_POSITION,
            "there is nothing above the position bar on the Now Playing page"
        );
    }

    /// Every shelf whose order is the user's is put in it, and the two whose
    /// order is their own are left alone.
    #[test]
    fn the_shelves_are_listed_in_the_order_that_was_asked_for() {
        let mut music = Music::new(None, true, true);
        let names = |music: &Music| -> Vec<String> {
            music.rows.iter().map(|row| row.label.clone()).collect()
        };

        music.change_tab(0);
        let forwards = names(&music);
        let mut sorted = forwards.clone();
        sorted.sort_by_key(|name| name.to_lowercase());
        assert_eq!(forwards, sorted, "Songs comes up by name");

        music.settings.order = Order::NameReversed;
        music.rebuild();
        let mut backwards = names(&music);
        backwards.reverse();
        assert_eq!(backwards, forwards, "and turns round when it is asked to");

        // The queue plays in the order it is going to play in, whatever the
        // shelves are set to.
        music.settings.order = Order::NameReversed;
        music.change_tab(4);
        let queued: Vec<usize> = music
            .rows
            .iter()
            .filter_map(|row| row.tracks.first().copied())
            .collect();
        assert_eq!(queued, music.queue.entries, "the queue was sorted");

        // And an album is disc and track number.
        music.change_tab(1);
        music.selected = 0;
        music.activate();
        music.crossing = None;
        assert!(music.group.is_some());
        let numbers: Vec<u32> = music
            .rows
            .iter()
            .filter_map(|row| row.tracks.first().map(|i| music.tracks[*i].number))
            .collect();
        let mut climbing = numbers.clone();
        climbing.sort_unstable();
        assert_eq!(
            numbers, climbing,
            "the album was sorted out of its own order"
        );
    }

    /// Changing the order keeps the light on the song somebody was looking at
    /// rather than on the place in the list it happened to be at.
    #[test]
    fn the_light_follows_the_song_across_a_change_of_order() {
        let mut music = Music::new(None, true, true);
        music.change_tab(0);
        music.selected = 3;
        let was = music.rows[3].tracks[0];
        music.sort_by(Order::NameReversed);
        assert_eq!(music.rows[music.selected].tracks[0], was);
    }

    /// The menu offers the six orders at its head, with the one in force
    /// ticked — and offers them only where an order means anything.
    #[test]
    fn the_menu_offers_an_order_where_an_order_means_something() {
        let mut music = Music::new(None, true, true);
        music.change_tab(0);

        // **One row, and the orders are behind it.** Not six bare nouns at the
        // head of a menu of acts, which is what this was reported as.
        let (rows, marked) = music.menu_rows();
        assert_eq!(rows[0], i18n::text("sort-by"), "sorting leads the menu");
        assert_eq!(marked, usize::MAX, "nothing in Options is an answer");
        for order in Order::ALL {
            assert!(
                !rows.contains(&order.label()),
                "an order is loose in the Options menu: {}",
                order.label()
            );
        }

        let (orders, marked) = music.order_rows();
        assert_eq!(orders.len(), Order::ALL.len());
        for (at, order) in Order::ALL.iter().enumerate() {
            assert_eq!(orders[at], order.label());
        }
        assert_eq!(marked, 0, "Name is the order it starts in, and is ticked");

        music.sort_by(Order::Oldest);
        let (_, marked) = music.order_rows();
        assert_eq!(marked, Order::ALL.len() - 1);

        // The queue plays in its own order, and an album is disc and track
        // number: neither is a shelf an order can be laid over, so neither is
        // offered one — and the row that would open them is not drawn either.
        music.change_tab(4);
        let (rows, _) = music.menu_rows();
        assert!(!music.can_sort(), "the queue was offered an order");
        assert!(!rows.contains(&i18n::text("sort-by")));

        music.change_tab(1);
        music.selected = 0;
        music.activate();
        music.crossing = None;
        let (rows, _) = music.menu_rows();
        assert!(!music.can_sort(), "an album was offered an order");
        assert!(!rows.contains(&i18n::text("sort-by")));
    }

    /// **Both menus fit the window this application opens at**, so that
    /// neither has an arrow at its foot with half its rows behind it.
    ///
    /// The one this replaced did not: six orders at the head of a menu of acts
    /// made twelve rows on the Songs shelf where ten fit, so two of them were
    /// behind an arrow. It is seven now. Asked of the toolkit's own sum, which
    /// is what decides the panel's window.
    ///
    /// Not asserted at 400 pixels, the shortest window this accepts: five rows
    /// fit there and nothing this menu could honestly say would come to five.
    /// A menu scrolls for that case, and this one is two rows over it rather
    /// than seven.
    #[test]
    fn neither_menu_has_to_be_scrolled_in_an_ordinary_window() {
        let fits = |rows: usize| {
            lxb_toolkit::menu::rows_of_that_fit(
                &vec![lxb_toolkit::menu::Row::command(1); rows],
                Some(1),
                800.0,
            )
        };
        let mut music = Music::new(None, true, true);
        for screen in [Screen::Library, Screen::Playing] {
            music.screen = screen;
            for tab in 0..TABS.len() {
                music.change_tab(tab);
                let (rows, _) = music.menu_rows();
                assert!(
                    fits(rows.len()) >= rows.len(),
                    "the Options menu is {} rows on shelf {tab} and only {} fit",
                    rows.len(),
                    fits(rows.len())
                );
            }
        }
        let (orders, _) = music.order_rows();
        assert!(fits(orders.len()) >= orders.len(), "the orders do not fit");
    }

    /// The row and the menu it opens are the same words, which is what says
    /// one is the other.
    #[test]
    fn the_row_that_opens_the_orders_names_them() {
        let mut music = Music::new(None, true, true);
        music.change_tab(0);
        let (rows, _) = music.menu_rows();
        let at = rows
            .iter()
            .position(|row| *row == i18n::text("sort-by"))
            .expect("there is a row for sorting");
        assert!(
            matches!(music.menu[at], Menu::Sorting),
            "the row named for sorting does something else"
        );
        // And every row of the menu it opens sets an order and nothing else.
        music.order_rows();
        for act in &music.menu {
            assert!(matches!(act, Menu::SortBy(_)));
        }
    }

    /// **Nothing in the menu closes the application.** Back does that, as it
    /// does in the film player and the photo viewer beside this one.
    #[test]
    fn no_row_of_the_menu_closes_the_application() {
        let mut music = Music::new(None, true, true);
        for tab in 0..TABS.len() {
            music.change_tab(tab);
            for screen in [Screen::Library, Screen::Playing] {
                music.screen = screen;
                let (rows, _) = music.menu_rows();
                assert_eq!(rows.len(), music.menu.len());
                assert!(!rows.is_empty(), "tab {tab} has an empty menu");
                for act in &music.menu {
                    // Exhaustive on purpose. A row that closed Music would
                    // have to be named here, which is where somebody would
                    // have to argue for it against the two applications
                    // beside this one, neither of which has one.
                    match act {
                        Menu::Sorting
                        | Menu::SortBy(_)
                        | Menu::Play
                        | Menu::Enqueue
                        | Menu::Favourite
                        | Menu::Unfavourite
                        | Menu::Remove
                        | Menu::AddFolder
                        | Menu::Refresh
                        | Menu::Clear => {}
                    }
                }
            }
        }
    }
}
