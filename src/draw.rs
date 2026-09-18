//! Music, composed: a rail of ways in, a browser, and the strip along the
//! bottom that is both what is playing and how to control it.
//!
//! Material, typography, metrics and focus are the toolkit's and are never
//! invented here. What this file owns is arithmetic: where things go, and how
//! one page grows out of the card that was pressed.
use crate::{
    app::{
        entry, Card, Change, Music, Row, Screen, Zone, BAR_LOUDNESS, BAR_POSITION, FIRST_BUTTON,
        NOW_PLAYING, RAIL_ADD_FOLDER, RAIL_OPTIONS, TABS, TRANSPORT,
    },
    i18n,
    legend::{self, hint, Button, Hint},
    library::time,
    message, playing,
    queue::Repeat,
};
use lxb_app::Page;
use lxb_render::{Align, Fit, Press, Selection, Ui};
use lxb_toolkit::{
    material::Surface,
    metrics::{capsule_radius, Metric},
    motion,
    palette::Role,
    settings::IconStyle,
    typography::Text,
};

/// What a pointer can land on. Every number here is well clear of
/// [`legend::SPOT`], and a row is numbered by the **place it was drawn in**
/// rather than by where it stands in the library — a list of a hundred
/// thousand songs numbers no more controls than a list of twenty.
pub const TAB: u32 = 100;
pub const CONTROL: u32 = 200;
pub const ADD_FOLDER: u32 = 300;
pub const OPTIONS: u32 = 301;
pub const HEADING: u32 = 302;
pub const STRIP: u32 = 303;
pub const VOLUME: u32 = 304;
pub const POSITION: u32 = 305;
/// The big button standing where the songs would be when there are none. It is
/// not the rail's own Add a folder and must not be numbered as one: two spots
/// sharing a number are two controls that cannot both be pressed, and a press
/// on this one would have sent the light off to the rail.
pub const ADD_FOLDER_HERE: u32 = 306;
pub const QUEUED: u32 = 400;
pub const ROW: u32 = 1000;

/// The mark a record with no sleeve of its own wears. It is the shell's own
/// mark for music, so a library with no artwork in it still looks like this
/// desktop rather than like a hole.
pub const RECORD: &str = "category-music";

/// The picture of a transport control, or `None` where the design language has
/// no honest one.
///
/// Seven of the nine are marks the toolkit already ships — the same
/// `media-previous`, `media-play`, `media-pause` and `media-next` the shell's
/// own media card draws, so pressing Play here and pressing Play in the guide
/// are plainly the same act. **Shuffle and repeat are not pictured**, because
/// there is no shuffle mark and no repeat mark in the language and the nearest
/// neighbours are both something else: `sort` is a sorting order and `refresh`
/// is reading a folder again. A word is better than the wrong mark — the same
/// rule the button legend keeps about Start — so those two are drawn as what
/// they are, two settings that say their own state.
fn transport_glyph(state: &Music, at: usize) -> Option<&'static str> {
    Some(match at {
        // The record, the two bars — rows, not chips — and the two settings.
        NOW_PLAYING | BAR_POSITION | BAR_LOUDNESS | 3 | 4 => return None,
        5 => "media-previous",
        6 => {
            if state.audio.status.playing {
                "media-pause"
            } else {
                "media-play"
            }
        }
        _ => "media-next",
    })
}

pub fn hints(state: &Music) -> Vec<Hint> {
    let accept = match (state.screen, state.zone) {
        (Screen::Playing, Zone::Queue) => i18n::text("play"),
        (Screen::Playing, _) => transport_label(state, state.transport),
        (_, Zone::Sidebar) => match state.rail {
            RAIL_ADD_FOLDER => i18n::text("add-folder"),
            RAIL_OPTIONS => i18n::text("options"),
            _ => i18n::text("open"),
        },
        (_, Zone::Transport) => transport_label(state, state.transport),
        (_, _) if state.rows.is_empty() => i18n::text("add-folder"),
        (_, _) if state.group.is_none() && (state.tab == 1 || state.tab == 2) => i18n::text("open"),
        _ => i18n::text("play"),
    };
    // Close only where the next Back really does close it. Anywhere it still
    // has somewhere to step back to — off the Now Playing page, out of an
    // album, off the browser to the rail — it says Back, because that is what
    // it does there.
    let back = if state.screen == Screen::Playing
        || state.group.is_some()
        || state.zone != Zone::Sidebar
    {
        i18n::text("back")
    } else {
        i18n::text("close")
    };
    vec![
        hint(accept, Button::Accept),
        hint(i18n::text("options"), Button::Options),
        hint(back, Button::Back),
        hint(
            if state.audio.status.playing {
                i18n::text("pause")
            } else {
                i18n::text("play")
            },
            Button::Start,
        ),
    ]
}

/// What a transport control is called. Read by the button legend, which is
/// words beside a picture of the button, and written on the two controls that
/// have no mark.
pub fn transport_label(state: &Music, at: usize) -> &'static str {
    i18n::text(match at {
        // A bar is named by what pressing it does, because that is what the
        // legend is for. What the directions do to it needs no word: the
        // handle moving is the word.
        BAR_POSITION => {
            if state.audio.status.playing {
                "pause"
            } else {
                "play"
            }
        }
        BAR_LOUDNESS => {
            if state.settings.volume > 0.0 {
                "mute"
            } else {
                "unmute"
            }
        }
        // The record at the head of the strip is never lettered — the song's
        // own name is written on it — so this is only ever read by the legend,
        // which says what South does to whatever the light is on.
        NOW_PLAYING => "open-now-playing",
        3 => {
            if state.queue.shuffle {
                "shuffle-on"
            } else {
                "shuffle-off"
            }
        }
        4 => match state.queue.repeat {
            Repeat::Off => "repeat-off",
            Repeat::All => "repeat-all",
            Repeat::One => "repeat-one",
        },
        5 => "previous",
        6 => {
            if state.audio.status.playing {
                "pause"
            } else {
                "play"
            }
        }
        _ => "next",
    })
}

/// The band across the foot of the window that belongs to the button legend,
/// and to nothing else.
///
/// **Measured in the toolkit's units rather than in this file's.** Everything
/// else on a page here is laid out against `s`, which is the window's height
/// over eight hundred; the row of hints is drawn by `legend::row` against the
/// design language's own scale, which is the window's height over ten hundred
/// and eighty and is clamped at both ends. A band worked out one way and a row
/// drawn the other way do not stay together as the window changes size: at the
/// size this opens at the two came out a pixel apart, which is what "the
/// helper buttons nearly touch the interface" was, and at a short window the
/// row hung off the bottom edge because the band had shrunk past the clamp and
/// the row had not.
///
/// Sixty-four is what Videos gives the same row, so the two applications' feet
/// line up when they are open one after the other.
pub const FOOT: f32 = 64.0;

/// How tall that band is in this window, in real pixels.
pub fn foot(ui: &Ui) -> f32 {
    ui.s(FOOT)
}

/// Where the row of hints sits inside that band: the middle of it.
///
/// A function rather than a sum written out at the one place it is needed,
/// because the band and the row have to be checked against each other and a
/// test that re-derives the placement is a test that passes whatever the page
/// really does.
pub fn legend_at(height: f32, foot: f32) -> f32 {
    height - foot * 0.5
}

/// A rectangle part of the way from one to another.
pub fn between(from: [f32; 4], to: [f32; 4], at: f32) -> [f32; 4] {
    let mut out = [0.0; 4];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = from[index] + (to[index] - from[index]) * at;
    }
    out
}

/// A name cut down until it fits the room it was given.
///
/// **Nothing clips a word.** `lxb-render` lays a label out at the width of its
/// rectangle only where the rectangle can hold more than one line, so every
/// one-line title is shaped unbounded and drawn straight out through whatever
/// stands beside it. So it is cut here, before it is handed over, by asking
/// the measurer rather than by counting letters: a Chinese character is a
/// whole em where a Latin one is about two thirds of one, and a count of
/// letters is a guess that is wrong in one language out of two.
///
/// Out of the **tail**, unlike a file name, which is cut out of the middle to
/// keep its extension: a song is prose, and prose is read from the front.
pub fn fit(ui: &mut Ui, style: Text, text: &str, room: f32) -> String {
    if room <= 0.0 {
        return String::new();
    }
    if ui.measure(style, text) <= room {
        return text.to_owned();
    }
    let letters: Vec<char> = text.chars().collect();
    let shortened = |keep: usize| {
        let mut cut: String = letters[..keep].iter().collect();
        while cut.ends_with(' ') {
            cut.pop();
        }
        cut.push('…');
        cut
    };
    let (mut least, mut most) = (0usize, letters.len());
    while least < most {
        let middle = least + (most - least).div_ceil(2);
        if ui.measure(style, &shortened(middle)) <= room {
            least = middle;
        } else {
            most = middle - 1;
        }
    }
    if least == 0 {
        String::from("…")
    } else {
        shortened(least)
    }
}

fn label(ui: &mut Ui, rect: [f32; 4], style: Text, text: &str, role: Role) {
    let cut = fit(ui, style, text, rect[2]);
    ui.label(rect, style, &cut, role, Align::Left);
}

fn label_faded(ui: &mut Ui, rect: [f32; 4], style: Text, text: &str, role: Role, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let cut = fit(ui, style, text, rect[2]);
    let tint = ui.tinted(role, alpha);
    ui.label_tinted(rect, style, &cut, tint, Align::Left);
}

/// A record sleeve, or the mark of one where there is none.
///
/// `opacity` is what a sleeve changing crosses over: the one going out is
/// drawn first at what is left of it, and the one coming in over the top of
/// it. Two halves of two different pictures do add up, which is right —
/// they are two pictures — where two halves of the *same* picture would not.
pub fn sleeve(
    ui: &mut Ui,
    rect: [f32; 4],
    path: Option<&std::path::Path>,
    icons: IconStyle,
    opacity: f32,
) {
    if opacity <= 0.0 || rect[2] <= 0.0 || rect[3] <= 0.0 {
        return;
    }
    let radius = ui.m(Metric::CardRadius) - ui.s(2.0);
    ui.card(rect, Surface::Control, Role::Accent, 0.28 * opacity);
    if path.is_some_and(|path| ui.picture(rect, radius, path, Fit::Cover, opacity)) {
        return;
    }
    let [x, y, w, h] = rect;
    let size = w.min(h) * 0.42;
    ui.icon_tinted(
        [x + (w - size) * 0.5, y + (h - size) * 0.5, size, size],
        RECORD,
        icons,
        Role::Text,
        0.75 * opacity,
    );
}

/// What a control under the light looks like right now: lit, and part of the
/// way through a dip if something has just been pressed.
pub fn press(state: &Music, lit: bool) -> Press {
    match state.press_through() {
        Some(through) if lit => Press::Going(through),
        _ => Press::from(lit),
    }
}

/// What shape the light under a chosen thing is.
///
/// **The light is the shape of the thing it is on, and is always said
/// outright.** The two roundings in this design language are not the same
/// number and never converge: a card is rounded to `Metric::CardRadius`, which
/// is a count of pixels, and a chip is rounded to half its own height, which
/// is however tall it happens to be. `Ui::selection` is the second of those
/// with no way to ask for the first, so lighting a row with it drew a
/// long pill over a rounded rectangle and left the card's own corners showing
/// outside the light — one thing wearing two shapes, which is what "the
/// highlight is not the shape of the button" was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lit {
    /// A card, a row in a list, a tab on the rail.
    Card,
    /// A chip: the shape `Ui::control` cuts and `Ui::button` wears.
    Chip,
}

/// Lay the light down.
///
/// **Before anything it is under, and never on the chosen one's turn in the
/// loop.** The light glides from what it was on to what it is going to, so for
/// the length of that glide it stands across both — and a light drawn when the
/// loop reaches the chosen row is a light drawn over every row above it, whose
/// sleeve is already down. A record's cover disappearing under the highlight
/// on the way past is exactly that, and so is a transport mark going out as
/// the light passes it. Every quad of a layer is drawn in the order it was
/// asked for, so the only place a background can be asked for is first.
///
/// It is also what `Ui::control` expects: it reads the light to work out how
/// far it has arrived over each chip, and a chip asked before the light was
/// placed is a chip that cannot know.
pub fn focus(ui: &mut Ui, light: &mut Selection, rect: [f32; 4], shape: Lit) {
    let at = light.glide(rect, ui.dt());
    // Off the rectangle the light has really glided to rather than off the one
    // it is going to: half of a height that is still moving is the only
    // rounding that stays a chip all the way across.
    let radius = match shape {
        Lit::Card => ui.m(Metric::CardRadius),
        Lit::Chip => capsule_radius(at[3]),
    };
    ui.lit(at, radius, lxb_toolkit::control::LIT_ROLE, at[2], 1.0);
}

/// How a page is being painted: the marks' style, the scale everything is
/// measured against, and whether this is the page that answers presses.
///
/// One value rather than three parameters, because all three are handed down
/// unchanged through every function here and the list had grown longer than
/// the arithmetic.
#[derive(Clone, Copy)]
pub struct Paint {
    pub icons: IconStyle,
    pub s: f32,
    /// During a crossing there are two pages on the screen and **neither**
    /// answers a press: both would write their targets down over each other.
    pub live: bool,
}

impl Paint {
    /// The same paint, for a page that is only being looked at.
    pub fn dead(self) -> Self {
        Self {
            live: false,
            ..self
        }
    }
}

/// Where drawing put the things a press has to be able to name.
///
/// Only drawing knows, and it is written down again every frame, so a window
/// resized while a page is open still shrinks back into the right card.
#[derive(Default)]
pub struct Placed {
    pub cards: Vec<Card>,
    pub queued: Vec<Card>,
    pub strip: Card,
    /// The two grooves. A click on a groove says a level outright, and only
    /// the rectangle it was drawn in can turn a point into one.
    pub volume: [f32; 4],
    pub position: [f32; 4],
}

/// One page of the browser: what it holds, what it is called, and where the
/// light is on it.
///
/// Passed about rather than read off the state, because during a crossing
/// there are two of them on the screen at once and only one of them is the
/// state.
pub struct View<'a> {
    pub rows: &'a [Row],
    pub title: &'a str,
    pub subtitle: &'a str,
    pub selected: usize,
    pub grid: bool,
    pub group: bool,
    pub cover: Option<&'a std::path::Path>,
    /// Whether this page draws its own heading sleeve. False where a crossing
    /// is growing one out of a card into exactly that place.
    pub sleeve: bool,
    /// How long this page has been on the screen, which is what its rows come
    /// in one after another from.
    pub since: f32,
}

/// The whole frame.
pub fn draw(state: &mut Music, page: &mut Page) {
    let width = page.width();
    let height = page.height();
    let icons = page.icons();
    let pad = page.pad_in_hand();
    let hints = hints(state);
    let seconds = state.seconds;
    let whole = [0.0, 0.0, width, height];
    let ui = page.ui();
    let s = height / 800.0;

    let mut light = state.light;
    let mut placed = Placed::default();
    let paint = Paint {
        icons,
        s,
        live: state.crossing.is_none(),
    };

    match &state.crossing {
        None => match state.screen {
            Screen::Library => {
                let view = live_view(state);
                library(state, ui, &mut light, &view, whole, paint, &mut placed);
            }
            Screen::Playing => {
                playing::page(state, ui, &mut light, whole, paint, true, &mut placed);
            }
        },
        Some(crossing) => {
            let grown = crossing.grown(seconds);
            let arriving = crossing.arriving(seconds);
            let stage = between(crossing.from, whole, grown);

            // The page underneath, which stays on the screen and steps back —
            // the gesture a page makes when a menu opens over it, at the size
            // of a whole change of page. `recede_behind`'s second half is the
            // important one: it cuts the words out from under what is standing
            // over them, and without it a card's name is drawn straight
            // through the page growing out of that very card.
            let under = under_view(state, crossing);
            library(state, ui, &mut light, &under, whole, paint, &mut placed);
            ui.recede(grown, crossing.from);
            // `dim` is what is *left* of the page behind, not how much is
            // taken off it. It follows the **rectangle** rather than the page
            // coming in: a page here is glass over the wallpaper rather than a
            // sheet that fills the window, so clearing what is behind it three
            // times as fast leaves most of the screen bare wallpaper half way
            // through. `amount` is the text cut and does follow the page,
            // because that is how solid the thing standing over the words
            // really is.
            ui.recede_behind(stage, 1.0 - grown, arriving);

            // The page over it, laid out at the size of the **window** and
            // moved, never laid out into the growing rectangle: type cannot be
            // scaled, so a page laid out small is a page whose words overlap.
            // Which means the sleeve's landing place is worked out against
            // where the page will be, not against the rectangle it shows
            // through. Then clipped to the stage and faded up as one thing —
            // `fade_between` is the only way a page can fade its own glass,
            // because a glass quad's material is not scaled by its tint.
            let offset = [
                stage[0] + stage[2] * 0.5 - width * 0.5,
                stage[1] + stage[3] * 0.5 - height * 0.5,
            ];
            let moved = [offset[0], offset[1], width, height];
            // The same band the page it is growing into will stand on, so
            // that a sleeve lands exactly where the page puts it.
            let stand_in = standing_in(state, crossing, moved, s, foot(ui));
            let before = ui.written();
            // Neither page numbers a control while they are crossing.
            let mut ignored = Placed::default();
            match crossing.what {
                Change::Playing => {
                    playing::page(
                        state,
                        ui,
                        &mut light,
                        moved,
                        paint.dead(),
                        stand_in.is_none(),
                        &mut ignored,
                    );
                }
                Change::Group => {
                    let mut over = over_view(state, crossing);
                    over.sleeve = stand_in.is_none();
                    library(
                        state,
                        ui,
                        &mut light,
                        &over,
                        moved,
                        paint.dead(),
                        &mut ignored,
                    );
                }
            }
            let page_ends = ui.written();
            ui.fade_between(before, page_ends, arriving);

            // And the sleeve, which grows out of the card's own picture into
            // the page's. It is drawn **over** the page rather than under it —
            // a page is a sheet of glass, and a picture under one is a picture
            // behind a window — and at full strength rather than crossing with
            // anything: two halves of one picture fading past each other add
            // up to three quarters, and the quarter missing is the wallpaper
            // showing through.
            if let Some((rect, cover)) = stand_in {
                sleeve(ui, rect, cover, icons, 1.0);
            }
            ui.cut_between(before, ui.written(), stage);
        }
    }

    // The row of button hints is the one thing that is never crossed over. It
    // is the same row in the same place on every page here and in the shell,
    // so it is the frame rather than the page: drawn after everything, and so
    // neither receded with the page behind nor clipped to the page in front.
    if lxb_toolkit::settings::button_hints().unwrap_or(true) {
        let middle = legend_at(height, foot(ui));
        legend::row(ui, width - 24.0 * s, middle, &hints, pad, icons);
    }

    state.light = light;
    state.placed(placed);
}

fn live_view(state: &Music) -> View<'_> {
    View {
        rows: &state.rows,
        title: "",
        subtitle: "",
        selected: state.selected,
        grid: state.grid(),
        group: state.group.is_some(),
        cover: state
            .group
            .as_ref()
            .and_then(|_| state.rows.iter().find_map(|row| row.cover.as_deref())),
        sleeve: true,
        since: state.seconds - state.arrived,
    }
}

/// The page standing underneath a crossing.
///
/// Going into an album it is the wall the album was on, which the crossing is
/// carrying; coming back out of one it is the wall, which is the state again.
/// Either way it is the page that is *not* moving.
fn under_view<'a>(state: &'a Music, crossing: &'a crate::app::Crossing) -> View<'a> {
    // Settled either way. The page underneath is the one that is *not* moving:
    // going in it has been there all along, and coming back out it is being
    // uncovered rather than fetched. A page that re-runs its entrance under a
    // page shrinking into it is two entrances on one screen.
    match (crossing.what, crossing.back) {
        (Change::Group, false) => View {
            rows: &crossing.leaving,
            title: "",
            subtitle: "",
            selected: state.selected,
            grid: state.tab == 1,
            group: false,
            cover: None,
            sleeve: true,
            since: f32::MAX,
        },
        _ => View {
            since: f32::MAX,
            ..live_view(state)
        },
    }
}

/// The page standing over a crossing: the songs on the album, whichever way
/// the crossing is going.
fn over_view<'a>(state: &'a Music, crossing: &'a crate::app::Crossing) -> View<'a> {
    let (rows, title, subtitle) = if crossing.back {
        (
            &crossing.leaving,
            crossing.title.as_str(),
            crossing.subtitle.as_str(),
        )
    } else {
        (&state.rows, "", "")
    };
    View {
        rows,
        title,
        subtitle,
        selected: if crossing.back { 0 } else { state.selected },
        grid: false,
        group: true,
        cover: rows.iter().find_map(|row| row.cover.as_deref()),
        sleeve: true,
        // The page coming in arrived when the crossing began, so its rows come
        // in one after another *as* it grows rather than all at once once it
        // has landed.
        since: state.seconds - state.arrived,
    }
}

/// The sleeve that grows out of the card while a page opens, and the
/// rectangle it is at right now.
///
/// `None` where there is nothing to grow — an album page has a small sleeve
/// beside its heading and the Now Playing page has a large one, and a card
/// with neither a picture nor a mark to put on the stage is better left to the
/// page itself.
fn standing_in<'a>(
    state: &'a Music,
    crossing: &'a crate::app::Crossing,
    moved: [f32; 4],
    s: f32,
    foot: f32,
) -> Option<([f32; 4], Option<&'a std::path::Path>)> {
    if crossing.art[2] <= 0.0 {
        return None;
    }
    let grown = crossing.grown(state.seconds);
    let landed = match crossing.what {
        Change::Playing => playing::sleeve_rect(moved, s, foot),
        Change::Group => heading_sleeve(moved, s),
    };
    let cover = match crossing.what {
        Change::Playing => state.sleeve(),
        Change::Group => crossing
            .leaving
            .iter()
            .chain(state.rows.iter())
            .find_map(|row| row.cover.as_deref()),
    };
    Some((between(crossing.art, landed, grown), cover))
}

// ---- where the things that can be chosen go ----------------------------
//
// Four sums, each written once and read twice: by the part of the page that
// draws the thing, and by [`light_on`] before any part of the page has drawn
// anything. Two answers would drift, and the drift would show as a highlight
// standing somewhere the thing under it is not.

/// Where a tab on the rail goes.
fn tab_at(at: [f32; 4], s: f32, since: f32, index: usize) -> [f32; 4] {
    let [ox, oy, ..] = at;
    [
        ox + 34.0 * s,
        oy + 106.0 * s + (54.0 + index as f32 * 56.0) * s + (1.0 - entry(since, index)) * 14.0 * s,
        164.0 * s,
        48.0 * s,
    ]
}

/// Where a row of the rail goes: the five shelves, and then the two buttons
/// standing at the foot of it.
///
/// One sum for all seven, because all seven are rows the light walks down and
/// the light has to be told where each of them is before the page is drawn.
fn rail_at(at: [f32; 4], s: f32, since: f32, bottom: f32, row: usize) -> [f32; 4] {
    if row < TABS.len() {
        return tab_at(at, s, since, row);
    }
    let [ox, ..] = at;
    let up = if row == RAIL_ADD_FOLDER { 112.0 } else { 60.0 };
    [ox + 36.0 * s, bottom - up * s, 160.0 * s, 44.0 * s]
}

/// How tall a row of the list is and how many of them the browser holds.
fn rows_in(body: [f32; 4], s: f32) -> (f32, usize) {
    let row_h = 62.0 * s;
    (
        row_h,
        ((body[3] - 30.0 * s) / row_h).floor().max(1.0) as usize,
    )
}

/// Where a row of the list goes.
fn row_at(body: [f32; 4], s: f32, since: f32, slot: usize, dip: Option<f32>) -> [f32; 4] {
    let [x, y, w, _] = body;
    let (row_h, _) = rows_in(body, s);
    let lift = (1.0 - entry(since, slot)) * 20.0 * s;
    let ry = y + 26.0 * s + slot as f32 * row_h + lift;
    motion::pressed([x, ry, w, row_h - 4.0 * s], dip)
}

/// How the wall of records is divided up: columns, the size of a cell, and how
/// many cells the browser holds.
fn cells_in(body: [f32; 4], s: f32) -> (usize, [f32; 2], usize) {
    let [_, _, w, h] = body;
    let gap = 18.0 * s;
    let columns = ((w / (210.0 * s)).floor() as usize).clamp(2, 7);
    let cell_w = (w - gap * (columns - 1) as f32) / columns as f32;
    let cell_h = cell_w + 62.0 * s;
    let lines = ((h + gap) / (cell_h + gap)).floor().max(1.0) as usize;
    (columns, [cell_w, cell_h], lines * columns)
}

/// Where a record on the wall goes.
fn card_at(body: [f32; 4], s: f32, since: f32, slot: usize, dip: Option<f32>) -> [f32; 4] {
    let [x, y, ..] = body;
    let gap = 18.0 * s;
    let (columns, [cell_w, cell_h], _) = cells_in(body, s);
    let lift = (1.0 - entry(since, slot)) * 22.0 * s;
    motion::pressed(
        [
            x + (slot % columns) as f32 * (cell_w + gap),
            y + (slot / columns) as f32 * (cell_h + gap) + lift,
            cell_w,
            cell_h,
        ],
        dip,
    )
}

/// The column of the strip the song's name is written in: where it starts and
/// how wide it is. Everything else on the strip is hung off its right-hand
/// edge.
fn strip_words(strip: [f32; 4], s: f32) -> (f32, f32) {
    let [x, _, w, h] = strip;
    let pad = 14.0 * s;
    let at = x + pad + (h - pad * 2.0) + 16.0 * s;
    // **A floor is only a floor while there is a floor to stand on.** This
    // wants a share of the strip and at least 150 of them, but 150 at this
    // scale can be more than the strip has left after the sleeve — a tall,
    // narrow window is a big `s` and a small `w` at once — and what it took
    // beyond that came out of the bars and the buttons beside it, which had
    // no say in the matter. Half of what is left after the sleeve, at most:
    // the name of a song and the controls for it are not a thing and its
    // margin, they are two halves of one strip.
    let room = (x + w - pad - at).max(0.0);
    (at, (w * 0.24).clamp(150.0 * s, 300.0 * s).min(room * 0.5))
}

/// Where the strip's two bars stand: how far through the song, and how loud,
/// one above the other beside the name.
///
/// **A row apiece, not a groove apiece.** The light stands on a bar the way it
/// stands on a row of a list, and a highlight drawn round a line four pixels
/// tall would be a thread rather than a selection. The row is the target a
/// pointer lands on too, so aiming at a bar and missing it by the height of a
/// clock still says a place along it.
///
/// The loudness bar is the shorter of the two. It is set by eye and lives
/// beside a mark that says which way is louder; the position bar is read as
/// well as set, and wants every pixel it can have for a song an hour long.
fn bars_at(strip: [f32; 4], s: f32) -> [[f32; 4]; 2] {
    let [x, y, w, _] = strip;
    let pad = 14.0 * s;
    let (words_x, words_w) = strip_words(strip, s);
    let left = words_x + words_w + 14.0 * s;
    let across = (x + w - pad - left).max(1.0);
    let line = y + pad + 2.0 * s;
    let tall = 30.0 * s;
    [
        [left, line, across, tall],
        [
            left,
            line + tall + 4.0 * s,
            (across * 0.45).clamp(160.0 * s, 340.0 * s).min(across),
            tall,
        ],
    ]
}

/// How far inside its own row a bar draws.
///
/// The same number a row of the list keeps its sleeve from the edge of the row,
/// and the same one the record at the head of the strip keeps from the edge of
/// the strip. Nothing in this interface is drawn flush to the thing it is in.
///
/// A row is taller than what it holds, so there was always air above and below
/// whatever a bar put in it — and none at all at the ends, where the loudness
/// bar's mark came up hard against the left-hand edge of its own highlight.
/// A selection with the control halfway out of it reads as a control that has
/// slipped, which is exactly what it was reported as.
const BAR_INSET: f32 = 7.0;

/// The room a bar has to draw in, inside the row that the light and the
/// pointer both get.
///
/// The row is what is lit and what is aimed at, and stays the size it was:
/// making the target smaller to give the drawing air would be paying for the
/// look of the thing with the ease of hitting it.
fn bar_inside(row: [f32; 4], s: f32) -> [f32; 4] {
    let [x, y, w, h] = row;
    let inset = BAR_INSET * s;
    [x + inset, y, (w - inset * 2.0).max(1.0), h]
}

/// Where the record at the head of the strip stands: the sleeve and the two
/// lines beside it, which together are what is playing and the way back to it.
///
/// One sum, read by the strip as it draws it, by the light before the page is
/// drawn at all, and by the crossing, which grows the Now Playing page out of
/// exactly this rectangle.
pub fn now_playing_at(strip: [f32; 4], s: f32) -> [f32; 4] {
    let [x, y, _, h] = strip;
    let (words_x, words_w) = strip_words(strip, s);
    // **Round the sleeve as well as the words.** The highlight is what says
    // this row is a thing you can press, and a highlight that cut across the
    // record at two thirds of its height looked like the record was sticking
    // out of the bottom of it. Which is why the row of buttons was moved out
    // from under the name: everything to the right of this block is the
    // column of three the light walks down, and nothing else shares its room.
    let inset = 7.0 * s;
    [
        x + inset,
        y + inset,
        (words_x + words_w + inset - (x + inset)).max(1.0),
        (h - inset * 2.0).max(1.0),
    ]
}

/// Where the row of five transport buttons sits inside the strip.
///
/// Under the two bars and lined up with them, rather than under the name: the
/// record at the head of the strip is a row the light stands on now, and it
/// takes the whole of the left-hand block for its own.
fn controls_at(strip: [f32; 4], s: f32) -> [f32; 4] {
    let [x, y, w, h] = strip;
    let pad = 14.0 * s;
    let left = bars_at(strip, s)[0][0];
    let row_h = 46.0 * s;
    [
        left,
        y + h - pad - row_h,
        (x + w - pad - left).max(1.0),
        row_h,
    ]
}

/// Where each of the five goes in that row.
///
/// Not evenly: the two settings say their own state and need room for a word,
/// and the three that are marks want to be round. The marks are centred on the
/// whole width rather than pushed along by whatever the two words beside them
/// measure in this language, so the row reads as two groups — what is on, and
/// what is playing.
pub fn transport_at(at: [f32; 4], s: f32) -> Vec<[f32; 4]> {
    let [x, y, w, h] = at;
    let gap = 8.0 * s;
    // What the row would like: three round marks off its own height, and two
    // chips wide enough to hold a word.
    let marks_want = h * 1.28;
    let chips_want = (w * 0.155).clamp(112.0 * s, 168.0 * s);
    // **Both groups give up the same share of what they wanted.** This row is
    // measured in `s`, which comes off the window's *height*, so a tall narrow
    // window asks for a row far wider than it has given — and the answer has
    // to be five smaller controls rather than three full-sized ones and two
    // that have quietly become nothing.
    //
    // The gaps are not squeezed with them: a gap that shrinks along with
    // everything else keeps nothing apart at all. There are four — one between
    // the two words, one between the groups, and two between the three marks.
    let gaps = gap * 4.0;
    let want = marks_want * 3.0 + chips_want * 2.0;
    let squeeze = if want > 0.0 {
        ((w - gaps) / want).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mark_w = (marks_want * squeeze).max(1.0);
    let chip_w = chips_want * squeeze;
    let marks_w = mark_w * 3.0 + gap * 2.0;
    // And the marks centred on the **whole** row wherever the row can afford
    // it, so that it reads as two groups — what is on, and what is playing —
    // rather than as five things shoved along by however long "Shuffle off"
    // happens to be in this language. Pushed clear of the words where it
    // cannot: that is the one thing centring may never cost.
    let after = x + chip_w * 2.0 + gap * 2.0;
    let marks_x = (x + (w - marks_w) * 0.5).clamp(after, (x + w - marks_w).max(after));
    (0..TRANSPORT - FIRST_BUTTON)
        .map(|slot| match slot {
            0 => [x, y, chip_w, h],
            1 => [x + chip_w + gap, y, chip_w, h],
            _ => [marks_x + (slot - 2) as f32 * (mark_w + gap), y, mark_w, h],
        })
        .collect()
}

/// Which row of the browser is the first one drawn, given which one is chosen.
///
/// Asked of the two numbers rather than of the whole `View`, because what it
/// has to keep true — that the chosen row is one of the rows really drawn — is
/// a fact about those two and a window size, and is tested as one.
fn first_shown(selected: usize, grid: bool, body: [f32; 4], s: f32) -> usize {
    if grid {
        let (columns, _, shown) = cells_in(body, s);
        (selected / columns).saturating_sub(shown / columns - 1) * columns
    } else {
        let (_, shown) = rows_in(body, s);
        selected.saturating_sub(shown - 1)
    }
}

/// Where the one light on this page stands, and what shape it is.
///
/// **One light for the whole screen, and the screen lays it down before it
/// draws anything.** The rail, the browser and the transport hand a single
/// `Selection` between them, so for as long as it is gliding from one of them
/// to another it belongs to neither — and the part that drew it drew it over
/// everything the parts before it had already put down. A record's cover
/// swallowed by the highlight as it passes, and a transport mark going out as
/// the light leaves the list for the strip, are both that. Asked here and
/// drawn first, it is what it has to be: the background of whatever it is on,
/// with the whole page standing on top of it.
fn light_on(
    state: &Music,
    view: &View<'_>,
    at: [f32; 4],
    body: [f32; 4],
    strip: [f32; 4],
    paint: Paint,
) -> Option<([f32; 4], Lit)> {
    if !paint.live {
        return None;
    }
    let s = paint.s;
    // The dip a press puts the chosen thing through. It is part of where the
    // thing *is*, so the light travels down with it.
    let dip = press(state, true).through();
    match state.zone {
        Zone::Sidebar => {
            let row = state.rail;
            let rect = rail_at(at, s, state.seconds - state.opened, body[1] + body[3], row);
            // A shelf is a card; the two buttons at the foot are chips, which
            // is what `Ui::button` cuts them from.
            Some((
                rect,
                if row < TABS.len() {
                    Lit::Card
                } else {
                    Lit::Chip
                },
            ))
        }
        Zone::Transport => match state.transport {
            // The record at the head of the strip, and the two bars: all three
            // are rows, and wear a row's highlight.
            NOW_PLAYING => Some((now_playing_at(strip, s), Lit::Card)),
            BAR_POSITION => Some((bars_at(strip, s)[0], Lit::Card)),
            BAR_LOUDNESS => Some((bars_at(strip, s)[1], Lit::Card)),
            button => transport_at(controls_at(strip, s), s)
                .get(button - FIRST_BUTTON)
                .copied()
                .map(|rect| (rect, Lit::Chip)),
        },
        Zone::Library if !view.rows.is_empty() => {
            let slot = view
                .selected
                .checked_sub(first_shown(view.selected, view.grid, body, s))?;
            let rect = if view.grid {
                card_at(body, s, view.since, slot, dip)
            } else {
                row_at(body, s, view.since, slot, dip)
            };
            Some((rect, Lit::Card))
        }
        // Nothing on this screen is in the queue, and an empty browser has no
        // row to stand on — the button under it is the toolkit's own and
        // lights itself.
        _ => None,
    }
}

/// Where the small sleeve beside an album's heading goes.
fn heading_sleeve(view: [f32; 4], s: f32) -> [f32; 4] {
    let [x, y, ..] = view;
    let side = 84.0 * s;
    [
        x + 24.0 * s + 184.0 * s + 18.0 * s,
        y + 106.0 * s,
        side,
        side,
    ]
}

/// The library screen: the rail, the browser and the strip along the bottom.
///
/// Answers where it put the browser's rows and where it put the strip, so that
/// a press can name a rectangle. Only drawing knows.
#[allow(clippy::too_many_arguments)]
fn library(
    state: &Music,
    ui: &mut Ui,
    light: &mut Selection,
    view: &View<'_>,
    at: [f32; 4],
    paint: Paint,
    placed: &mut Placed,
) {
    let (icons, s) = (paint.icons, paint.s);
    let [ox, oy, width, height] = at;
    let margin = 24.0 * s;
    let gap = 18.0 * s;
    let rail_w = 184.0 * s;
    let content_x = ox + margin + rail_w + gap;
    let content_w = width - margin * 2.0 - rail_w - gap;
    let top = oy + 106.0 * s;
    let strip_h = 148.0 * s;
    // The strip stands **on** the legend's band rather than across it, and the
    // browser stops a hair above the strip. One sum, read by the rail as well,
    // because a rail that worked its own bottom out separately is a rail whose
    // edge stops agreeing with the strip's the moment either number changes.
    let strip_y = oy + height - foot(ui) - strip_h;
    let bottom = strip_y - 10.0 * s;
    let strip = [ox + margin, strip_y, width - margin * 2.0, strip_h];
    let body = [
        content_x,
        top + 96.0 * s,
        content_w,
        bottom - (top + 96.0 * s),
    ];

    head(state, ui, at, s);

    // **The two sheets, then the light, then everything that stands on them.**
    // The light is the background of a *thing on a sheet*, not of the sheet,
    // so a light laid before the sheets is a light the rail's glass and the
    // strip's glass are drawn straight over — a chosen chip with no highlight
    // on it at all. Laid here it is above both sheets and beneath every tab,
    // row, sleeve and mark, whichever of them it is on and whichever two of
    // them it is between. See `light_on`.
    ui.card(
        [ox + margin, top, rail_w, bottom - top],
        Surface::Sidebar,
        Role::Glass,
        0.65,
    );
    ui.card(strip, Surface::Panel, Role::Glass, 0.65);
    // The shelf the rail is on wears a card of its own, and that card is a
    // background as much as the two sheets are: it says *this is where you
    // are*, which is true whether or not the rail has the light. Drawn in the
    // rail's own loop it was drawn over the light standing on the very same
    // tab — a highlight with a tab painted on top of it.
    let opened = state.seconds - state.opened;
    ui.card(
        tab_at(at, s, opened, state.tab),
        Surface::Control,
        Role::Accent,
        0.24 * entry(opened, state.tab),
    );
    if let Some((rect, shape)) = light_on(state, view, at, body, strip, paint) {
        focus(ui, light, rect, shape);
    }

    rail(state, ui, paint, at, bottom);

    let title = if view.title.is_empty() {
        state.title()
    } else {
        view.title.to_owned()
    };
    let subtitle = if view.subtitle.is_empty() {
        state.subtitle()
    } else {
        view.subtitle.to_owned()
    };
    let mut heading_x = content_x;
    let mut heading_w = content_w;
    if view.group {
        let art = heading_sleeve(at, s);
        if view.sleeve {
            sleeve(ui, art, view.cover, icons, 1.0);
        }
        heading_x = art[0] + art[2] + 16.0 * s;
        heading_w = content_w - (heading_x - content_x);
    }
    // What number in the list the light is on, beside the heading rather than
    // under the last row: a line under the list is a line inside the strip at
    // any window short enough to matter.
    let counter = (view.rows.len() > state.visible_rows).then(|| {
        message!(
            "position-in-list",
            "at" => view.selected + 1,
            "of" => view.rows.len(),
        )
    });
    let counter_w = counter
        .as_deref()
        .map(|counter| ui.measure(Text::Caption, counter) + 18.0 * s)
        .unwrap_or(0.0);
    if let Some(counter) = &counter {
        ui.label(
            [
                content_x + content_w - counter_w,
                top + 12.0 * s,
                counter_w,
                24.0 * s,
            ],
            Text::Caption,
            counter,
            Role::TextSoft,
            Align::Right,
        );
    }
    let heading_w = heading_w - counter_w;
    label(
        ui,
        [heading_x + 2.0 * s, top + 3.0 * s, heading_w, 40.0 * s],
        Text::Title,
        &title,
        Role::Text,
    );
    label(
        ui,
        [heading_x + 2.0 * s, top + 48.0 * s, heading_w, 28.0 * s],
        Text::Caption,
        &subtitle,
        Role::TextSoft,
    );
    if view.group && paint.live {
        ui.spot(HEADING, [content_x, top, content_w, 84.0 * s]);
    }

    let cards = if view.rows.is_empty() {
        nothing_here(state, ui, body, paint, view);
        Vec::new()
    } else if view.grid {
        wall(state, ui, view, body, paint)
    } else {
        list(state, ui, view, body, paint)
    };

    bar(state, ui, strip, paint, placed);
    if paint.live {
        placed.cards = cards;
    }
}

/// The name of the application and what is in the library, along the top.
fn head(state: &Music, ui: &mut Ui, at: [f32; 4], s: f32) {
    let [ox, oy, width, _] = at;
    let margin = 24.0 * s;
    label(
        ui,
        [ox + margin, oy + 20.0 * s, width * 0.5, 42.0 * s],
        Text::Title,
        i18n::text("app-name"),
        Role::Text,
    );
    label(
        ui,
        [ox + margin, oy + 64.0 * s, width * 0.5, 26.0 * s],
        Text::Caption,
        if state.demo {
            i18n::text("preview")
        } else {
            i18n::text("app-tagline")
        },
        Role::TextSoft,
    );
    let counter = if state.scanning {
        i18n::text("scanning").to_owned()
    } else {
        format!(
            "{}  ·  {}",
            message!("song-count", "count" => state.tracks.len()),
            message!("album-count", "count" => state.album_count),
        )
    };
    let room = width * 0.45;
    let cut = fit(ui, Text::Caption, &counter, room);
    ui.label(
        [ox + width - margin - room, oy + 44.0 * s, room, 28.0 * s],
        Text::Caption,
        &cut,
        Role::TextSoft,
        Align::Right,
    );
    if state.scanning {
        // A mark that breathes while the walk is still going, so that a long
        // library does not look like a stopped one.
        let alpha = 0.25 + 0.45 * motion::pulse(state.seconds);
        ui.glow(
            [
                ox + width - margin - 14.0 * s,
                oy + 44.0 * s,
                10.0 * s,
                10.0 * s,
            ],
            Role::Accent,
            alpha,
        );
    }
}

/// The rail down the left-hand side.
/// The rail down the left-hand side.
///
/// **Its own clock, and not the page's.** The rail is the frame rather than the
/// page: it is the same five rows before and after a shelf is chosen, so
/// reading the page's arrival would fade all five words out and back in every
/// time somebody moved between them — which is what "the sidebar text
/// disappears" was.
fn rail(state: &Music, ui: &mut Ui, paint: Paint, at: [f32; 4], bottom: f32) {
    let (s, live) = (paint.s, paint.live);
    let since = state.seconds - state.opened;
    let [ox, oy, ..] = at;
    let margin = 24.0 * s;
    let rail_w = 184.0 * s;
    let top = oy + 106.0 * s;
    label(
        ui,
        [
            ox + margin + 18.0 * s,
            top + 18.0 * s,
            rail_w - 36.0 * s,
            22.0 * s,
        ],
        Text::Caption,
        i18n::text("library"),
        Role::TextSoft,
    );
    for (index, tab) in TABS.iter().enumerate() {
        let arrived = entry(since, index);
        let rect = tab_at(at, s, since, index);
        label_faded(
            ui,
            [
                rect[0] + 14.0 * s,
                rect[1] + 10.0 * s,
                rect[2] - 28.0 * s,
                30.0 * s,
            ],
            Text::Label,
            i18n::text(tab),
            Role::Text,
            arrived,
        );
        if live {
            ui.spot(TAB + index as u32, rect);
        }
    }
    // The two buttons at the foot of the rail are rows of it, so they light
    // like every other row does — off `state.rail`, not off a pointer being
    // over them.
    let on = |row: usize| live && state.zone == Zone::Sidebar && state.rail == row;
    let add = rail_at(at, s, since, bottom, RAIL_ADD_FOLDER);
    ui.button(
        add,
        i18n::text("add-folder-short"),
        press(state, on(RAIL_ADD_FOLDER)),
    );
    let options = rail_at(at, s, since, bottom, RAIL_OPTIONS);
    ui.button(
        options,
        i18n::text("options"),
        press(state, on(RAIL_OPTIONS)),
    );
    if live {
        ui.spot(ADD_FOLDER, add);
        ui.spot(OPTIONS, options);
    }
}

/// What stands where the songs would be when there are none.
fn nothing_here(state: &Music, ui: &mut Ui, at: [f32; 4], paint: Paint, view: &View<'_>) {
    let (icons, s) = (paint.icons, paint.s);
    let [x, y, w, h] = at;
    let centre = y + h * 0.22;
    let arrived = entry(view.since, 0);
    let lift = (1.0 - arrived) * 18.0 * s;
    ui.icon_tinted(
        [x + 24.0 * s, centre + lift, 72.0 * s, 72.0 * s],
        RECORD,
        icons,
        Role::Text,
        0.85 * arrived,
    );
    let (heading, note) = if state.scanning {
        ("scanning", "scanning-note")
    } else if state.tracks.is_empty() {
        ("empty", "empty-note")
    } else if state.tab == 4 {
        ("empty-queue", "empty-queue-note")
    } else {
        ("empty-filter", "empty-note")
    };
    label_faded(
        ui,
        [
            x + 24.0 * s,
            centre + 96.0 * s + lift,
            w - 48.0 * s,
            40.0 * s,
        ],
        Text::Title,
        i18n::text(heading),
        Role::Text,
        arrived,
    );
    label_faded(
        ui,
        [
            x + 24.0 * s,
            centre + 142.0 * s + lift,
            w - 48.0 * s,
            30.0 * s,
        ],
        Text::Body,
        i18n::text(note),
        Role::TextSoft,
        arrived,
    );
    if paint.live && !state.scanning {
        let rect = [
            x + 24.0 * s,
            centre + 200.0 * s,
            (w - 48.0 * s).min(280.0 * s),
            48.0 * s,
        ];
        ui.button(rect, i18n::text("add-folder"), state.zone == Zone::Library);
        ui.spot(ADD_FOLDER_HERE, rect);
    }
}

/// A wall of record sleeves.
fn wall(state: &Music, ui: &mut Ui, view: &View<'_>, at: [f32; 4], paint: Paint) -> Vec<Card> {
    let (icons, s) = (paint.icons, paint.s);
    let (_, _, shown) = cells_in(at, s);
    let start = first_shown(view.selected, view.grid, at, s);
    let mut cards = Vec::new();
    for (slot, row) in view.rows.iter().skip(start).take(shown).enumerate() {
        let index = start + slot;
        let arrived = entry(view.since, slot);
        let lit = paint.live && view.selected == index && state.zone == Zone::Library;
        // The light this is under was laid down before the page — `card_at`
        // is read there as well, which is how the two agree about where a
        // record is.
        let rect = card_at(at, s, view.since, slot, press(state, lit).through());
        let [cx, cy, cell_w, _] = rect;
        let side = cell_w - 12.0 * s;
        let art = [cx + 6.0 * s, cy + 6.0 * s, side, side];
        sleeve(ui, art, row.cover.as_deref(), icons, arrived);
        label_faded(
            ui,
            [
                cx + 8.0 * s,
                cy + cell_w + 3.0 * s,
                cell_w - 16.0 * s,
                27.0 * s,
            ],
            Text::Label,
            &row.label,
            Role::Text,
            arrived,
        );
        label_faded(
            ui,
            [
                cx + 8.0 * s,
                cy + cell_w + 31.0 * s,
                cell_w - 16.0 * s,
                24.0 * s,
            ],
            Text::Caption,
            &row.detail,
            Role::TextSoft,
            arrived,
        );
        if paint.live {
            ui.spot(ROW + slot as u32, rect);
            cards.push(Card { index, rect, art });
        }
    }
    cards
}

/// A list of songs.
fn list(state: &Music, ui: &mut Ui, view: &View<'_>, at: [f32; 4], paint: Paint) -> Vec<Card> {
    let (icons, s) = (paint.icons, paint.s);
    let [x, y, w, _] = at;
    let (_, shown) = rows_in(at, s);
    let start = first_shown(view.selected, view.grid, at, s);
    label(
        ui,
        [x + 62.0 * s, y - 4.0 * s, w * 0.5, 23.0 * s],
        Text::Caption,
        i18n::text("track"),
        Role::TextSoft,
    );
    ui.label(
        [x + w - 66.0 * s, y - 4.0 * s, 58.0 * s, 23.0 * s],
        Text::Caption,
        i18n::text("length"),
        Role::TextSoft,
        Align::Right,
    );
    // **Only the row under the light wears anything.** This is a list of
    // songs, not a column of buttons: a card behind every row says that every
    // row is a thing to press, which is true of all of them and so tells the
    // eye nothing, and it leaves the one row that *is* chosen with no more to
    // say for itself than a change of tint. So the light *is* a row's
    // background — which is why it is laid down before the page and not on
    // the chosen row's turn here. See `light_on`.
    //
    // Striping every other one was tried before this and was worse still: the
    // eye reads the alternation as two kinds of row rather than as one list,
    // which is what it was reported as.
    let mut cards = Vec::new();
    for (slot, row) in view.rows.iter().skip(start).take(shown).enumerate() {
        let index = start + slot;
        let arrived = entry(view.since, slot);
        let chosen = paint.live && view.selected == index && state.zone == Zone::Library;
        let rect = row_at(at, s, view.since, slot, press(state, chosen).through());
        let ry = rect[1];
        let art = [x + 7.0 * s, ry + 6.0 * s, 44.0 * s, 44.0 * s];
        sleeve(ui, art, row.cover.as_deref(), icons, arrived);
        let playing = state
            .queue
            .current()
            .is_some_and(|current| row.tracks.len() == 1 && row.tracks[0] == current)
            && (row.queue_at.is_none() || row.queue_at == state.queue.position);
        // **The row the player is on is marked, not tinted.** It wore its name
        // in `Role::Accent`, which is a fill colour and not an ink: a
        // mid-saturation violet on a dark violet page, at caption weight, came
        // out as a title nobody could read — and on the row under the light it
        // would have been accent on accent. Nothing in this design language
        // says anything with the colour of type. So the name is the ink every
        // other name is, and what the row has that the others have not is the
        // transport's own mark, which says one thing more than a tint could:
        // whether the track is running or standing still.
        let mark = if playing { 30.0 * s } else { 0.0 };
        let words = w - 150.0 * s - mark;
        label_faded(
            ui,
            [x + 62.0 * s, ry + 5.0 * s, words, 28.0 * s],
            Text::Label,
            &row.label,
            Role::Text,
            arrived,
        );
        if playing {
            let side = 20.0 * s;
            ui.icon_tinted(
                [
                    x + w - 86.0 * s - side,
                    ry + (row_at(at, s, view.since, slot, None)[3] - side) * 0.5,
                    side,
                    side,
                ],
                if state.audio.status.playing {
                    "media-play"
                } else {
                    "media-pause"
                },
                icons,
                Role::Text,
                0.8 * arrived,
            );
        }
        let favourite = row.tracks.len() == 1
            && state
                .settings
                .favourites
                .contains(&state.tracks[row.tracks[0]].path);
        let mut detail = String::new();
        if playing {
            detail.push_str(i18n::text("current"));
            detail.push_str("  ·  ");
        }
        detail.push_str(&row.detail);
        if favourite {
            detail.push_str("  ·  ♥");
        }
        label_faded(
            ui,
            [x + 62.0 * s, ry + 32.0 * s, words, 23.0 * s],
            Text::Caption,
            &detail,
            Role::TextSoft,
            arrived,
        );
        let length: f64 = row.tracks.iter().map(|i| state.tracks[*i].duration).sum();
        let tint = ui.tinted(Role::TextSoft, arrived);
        ui.label_tinted(
            [x + w - 76.0 * s, ry + 19.0 * s, 68.0 * s, 25.0 * s],
            Text::Caption,
            &time(length),
            tint,
            Align::Right,
        );
        if paint.live {
            ui.spot(ROW + slot as u32, rect);
            cards.push(Card { index, rect, art });
        }
    }
    cards
}

/// The strip along the bottom: the sleeve, what it is, where it has got to,
/// and the nine controls.
///
/// The sleeve is the height of the whole strip, because it is also the card
/// the Now Playing page grows out of — a page opening out of a postage stamp
/// is a page that appears rather than one that opens.
pub fn bar(state: &Music, ui: &mut Ui, at: [f32; 4], paint: Paint, placed: &mut Placed) {
    let (icons, s, live) = (paint.icons, paint.s, paint.live);
    let [x, y, _, h] = at;
    let pad = 14.0 * s;
    let art_side = h - pad * 2.0;
    let art = [x + pad, y + pad, art_side, art_side];
    let fade = ((state.seconds - state.sleeve_at) / lxb_toolkit::motion::duration::COLOUR_FADE)
        .clamp(0.0, 1.0);
    sleeve(ui, art, state.sleeve(), icons, motion::smoothstep(fade));

    let current = state.current();
    let (words_x, words_w) = strip_words(at, s);
    let line = y + pad + 6.0 * s;
    label(
        ui,
        [words_x, line, words_w, 30.0 * s],
        Text::Label,
        current
            .map(|track| track.title.as_str())
            .unwrap_or(i18n::text("ready")),
        Role::Text,
    );
    label(
        ui,
        [words_x, line + 28.0 * s, words_w, 24.0 * s],
        Text::Caption,
        current
            .map(|track| track.artist.as_str())
            .unwrap_or(i18n::text("choose-song")),
        Role::TextSoft,
    );

    let length = current.map(|track| track.duration).unwrap_or(0.0);
    let position = if state.demo { 68.0 } else { state.position() };
    let [where_at, how_loud] = bars_at(at, s);
    let track = position_bar(ui, where_at, position, length, paint);
    let loud = volume(state, ui, how_loud, paint);

    let row = controls_at(at, s);
    controls(state, ui, row, paint);
    let card = now_playing_at(at, s);
    if live {
        ui.spot(STRIP, card);
        ui.spot(POSITION, where_at);
        placed.strip = Card {
            index: usize::MAX,
            rect: card,
            art,
        };
        placed.volume = loud;
        placed.position = track;
    }
}

/// How far through the song it is, as a bar: the two clocks at either end of a
/// groove. Answers where the groove went, so a press on the row can be turned
/// back into a place in the song.
///
/// The clocks are laid out first and the groove is given what is left over,
/// because a one-line label is shaped unbounded and two that overlap are two
/// that are both unreadable. A row too narrow to hold a groove between them
/// keeps the clocks and drops the groove: knowing where the song has got to
/// matters more than a line three pixels long.
pub fn position_bar(
    ui: &mut Ui,
    at: [f32; 4],
    position: f64,
    length: f64,
    paint: Paint,
) -> [f32; 4] {
    let s = paint.s;
    let [x, y, w, h] = bar_inside(at, s);
    let clock_w = 58.0 * s;
    let step = clock_w + 12.0 * s;
    let middle = y + h * 0.5;
    // **The length goes before the groove does.** How long the song is stands
    // on its own row in the listing as well; where it has got to is only ever
    // here. So a row that cannot hold two clocks and a groove worth the name
    // keeps the groove and the clock that is moving, and a row that cannot
    // hold even one of them keeps the two clocks — knowing where the song has
    // got to matters more than a line three pixels long, but it matters less
    // than being able to move it.
    let least = 60.0 * s;
    let both = w - step * 2.0 >= least;
    let lead = w - step >= least;
    ui.label(
        [x, middle - 12.0 * s, clock_w, 24.0 * s],
        Text::Caption,
        &time(position),
        Role::TextSoft,
        Align::Right,
    );
    if both || !lead {
        ui.label(
            [x + w - clock_w, middle - 12.0 * s, clock_w, 24.0 * s],
            Text::Caption,
            &time(length),
            Role::TextSoft,
            Align::Left,
        );
    }
    if !both && !lead {
        return [x, y, 0.0, 0.0];
    }
    let track = [
        x + step,
        middle - 2.0 * s,
        (w - step * if both { 2.0 } else { 1.0 }).max(0.0),
        4.0 * s,
    ];
    groove(ui, track, position, length, s);
    track
}

/// The nine controls, in one row.
///
/// Where each of them goes is [`transport_at`], because the light standing on
/// one of them was laid down before this page was drawn and had to be told
/// where. `Ui::control` reads that light to work out how far it has arrived
/// over the chip it is cutting, which is the other half of why it cannot be
/// placed here: a chip asked for before the light was put down is a chip that
/// cannot know.
pub fn controls(state: &Music, ui: &mut Ui, at: [f32; 4], paint: Paint) {
    let (icons, s, live) = (paint.icons, paint.s, paint.live);
    for (slot, rect) in transport_at(at, s).into_iter().enumerate() {
        let index = FIRST_BUTTON + slot;
        let chosen = live && state.zone == Zone::Transport && state.transport == index;
        let press = press(state, chosen);
        match transport_glyph(state, index) {
            // A mark on a chip of its own. `Ui::control` is the chip every
            // button in this language is cut from — asked for directly here
            // because `Ui::button` would put a word in it.
            Some(glyph) => {
                let radius = rect[3] * 0.5;
                let sunk = ui.control(rect, radius, press, rect[2], 1.0);
                let size = sunk[3] * 0.46;
                ui.icon_tinted(
                    [
                        sunk[0] + (sunk[2] - size) * 0.5,
                        sunk[1] + (sunk[3] - size) * 0.5,
                        size,
                        size,
                    ],
                    glyph,
                    icons,
                    Role::Text,
                    if chosen { 1.0 } else { 0.86 },
                );
            }
            // And the two the language has no mark for, saying what they are.
            None => {
                let label = transport_label(state, index);
                let cut = fit(ui, Text::Caption, label, rect[2] - 22.0 * s);
                ui.button(rect, &cut, press);
            }
        }
        if live {
            ui.spot(CONTROL + index as u32, rect);
        }
    }
}

/// How loud it is, as a groove rather than as a number.
///
/// The shape the shell's own quick settings use: the mark at the head and the
/// groove running off it, so the volume here and the volume in the guide are
/// plainly the same control. A pointer landing on the groove says a level
/// outright; the two marks beside it say a step.
pub fn volume(state: &Music, ui: &mut Ui, at: [f32; 4], paint: Paint) -> [f32; 4] {
    let (icons, s, live) = (paint.icons, paint.s, paint.live);
    let [x, y, w, h] = bar_inside(at, s);
    let mark = h.min(20.0 * s);
    ui.icon_tinted(
        [x, y + (h - mark) * 0.5, mark, mark],
        if state.settings.volume <= 0.0 {
            "volume-muted"
        } else {
            "volume"
        },
        icons,
        Role::TextSoft,
        0.9,
    );
    let track = [
        x + mark + 10.0 * s,
        y + (h - 4.0 * s) * 0.5,
        w - mark - 10.0 * s,
        4.0 * s,
    ];
    groove(ui, track, state.settings.volume as f64, 1.0, s);
    if live {
        ui.spot(VOLUME, at);
    }
    track
}

/// How far through the song it is.
pub fn groove(ui: &mut Ui, at: [f32; 4], position: f64, length: f64, s: f32) {
    let [x, y, w, h] = at;
    ui.chip([x, y, w, h], ui.tinted(Role::TextSoft, 0.35));
    if length <= 0.0 {
        return;
    }
    let through = (position / length).clamp(0.0, 1.0) as f32;
    if through > 0.0 {
        ui.chip([x, y, w * through, h], ui.tinted(Role::Accent, 1.0));
    }
    // The head of the groove: a bead of light on the line, which is how far
    // through it is at a glance rather than at a reading.
    let head = 9.0 * s;
    ui.glow(
        [
            x + w * through - head,
            y + h * 0.5 - head,
            head * 2.0,
            head * 2.0,
        ],
        Role::Accent,
        0.7,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_travels_from_one_to_the_other_and_stops() {
        let from = [10.0, 20.0, 30.0, 40.0];
        let to = [0.0, 0.0, 100.0, 200.0];
        assert_eq!(between(from, to, 0.0), from);
        assert_eq!(between(from, to, 1.0), to);
        let half = between(from, to, 0.5);
        assert!(half[2] > from[2] && half[2] < to[2]);
    }

    /// Every picture this transport asks for is a mark the toolkit really
    /// ships. A name that is not in the atlas draws **nothing at all** — an
    /// empty chip where a control was, with no message anywhere — so the seven
    /// are walked here the way the legend walks its own four.
    #[test]
    fn every_transport_mark_is_one_the_toolkit_has() {
        let mut music = Music::new(None, true, true);
        let mut pictured = 0;
        for playing in [true, false] {
            music.audio.status.playing = playing;
            for index in 0..TRANSPORT {
                if let Some(glyph) = transport_glyph(&music, index) {
                    assert!(
                        lxb_toolkit::assets::glyph(glyph).is_some(),
                        "the transport asks for a mark that is not in the toolkit: {glyph}"
                    );
                    pictured += 1;
                }
            }
        }
        assert_eq!(
            pictured, 6,
            "three of the five buttons are marks, under both faces of Play"
        );
        // A bar is a row and wears no chip, so it asks for no mark at all;
        // and the two settings are the two the language has no mark for.
        for bare in [BAR_POSITION, BAR_LOUDNESS, FIRST_BUTTON, FIRST_BUTTON + 1] {
            assert!(transport_glyph(&music, bare).is_none(), "{bare}");
        }
        assert!(lxb_toolkit::assets::glyph(RECORD).is_some());
    }

    /// Every one of the seven is named, and named something a catalog really
    /// carries — the legend writes whichever the light is standing on.
    #[test]
    fn every_transport_control_says_what_pressing_it_does() {
        let mut music = Music::new(None, true, true);
        for playing in [true, false] {
            music.audio.status.playing = playing;
            for volume in [0.0, 0.5] {
                music.settings.volume = volume;
                for index in 0..TRANSPORT {
                    let label = transport_label(&music, index);
                    assert!(!label.is_empty(), "{index} says nothing");
                    assert!(
                        !label.contains('-'),
                        "{index} came out as its own identifier: {label}"
                    );
                }
            }
        }
    }

    /// Nothing the pages number can land on the legend's own row, whatever
    /// happens to be in the library.
    #[test]
    fn no_control_number_can_reach_the_legends_own() {
        for base in [
            TAB, CONTROL, ADD_FOLDER, OPTIONS, HEADING, STRIP, QUEUED, ROW,
        ] {
            assert!(base < legend::SPOT);
        }
        // A page draws what fits on it, so the highest row it can number is
        // bounded by the rows it drew and not by the size of the library.
        const { assert!(ROW + 512 < legend::SPOT) };
    }

    /// The row of hints has a band of its own across the foot of the window,
    /// and the strip stands on it rather than across it, at every size the
    /// window is allowed to be.
    ///
    /// Both halves are read off the same two functions the page draws from —
    /// `FOOT` and `legend_at` here, `legend::GLYPH` there — because the fault
    /// this guards against was exactly the two halves being worked out from
    /// different scales and drifting into each other.
    #[test]
    fn the_button_hints_have_a_band_all_to_themselves() {
        for (w, h) in [
            (1280.0f32, 800.0f32),
            (960.0, 600.0),
            (640.0, 400.0),
            (3840.0, 2160.0),
            (1280.0, 400.0),
        ] {
            let scale = lxb_toolkit::metrics::scale_for(h);
            let foot = FOOT * scale;
            // Where `library` leaves the foot of the strip, which is the
            // lowest thing any page here draws.
            let strip_foot = h - foot;
            let middle = legend_at(h, foot);
            let glyph = legend::GLYPH * scale;
            assert!(
                middle - glyph * 0.5 > strip_foot,
                "{w}x{h}: the hints are drawn over the strip"
            );
            assert!(
                middle + glyph * 0.5 < h,
                "{w}x{h}: the hints hang off the bottom of the window"
            );
        }
    }

    /// The two roundings this design language has are different numbers at
    /// every size a row is drawn at here.
    ///
    /// Which is why [`Lit`] has to say which one it wants, and why
    /// `Ui::selection` — a chip's rounding with no way to ask for a card's —
    /// cannot be used on anything that is a card. Lighting a row with it drew
    /// a pill over a rounded rectangle and left the rectangle's corners
    /// showing outside the light.
    #[test]
    fn a_card_and_a_chip_are_never_the_same_shape() {
        for height in [400.0f32, 600.0, 800.0, 1080.0, 2160.0] {
            let s = height / 800.0;
            // A row in the list, which is the shape the fault was reported on.
            let row = 62.0 * s - 4.0 * s;
            let card = Metric::CardRadius.on(height);
            assert!(
                (capsule_radius(row) - card).abs() > 1.0,
                "{height}: a chip's rounding and a card's came out the same, \
                 so the difference this enum carries would be invisible"
            );
        }
    }

    /// The light never stands on a row the browser did not draw.
    ///
    /// `light_on` asks `first_shown` where the browser starts and then hands
    /// the difference to `row_at` or `card_at` as a slot. If the chosen row
    /// were outside the window those two draw, the highlight would be laid
    /// down on empty page — off the bottom of a short window, or in the
    /// column after the last record.
    #[test]
    fn the_light_only_ever_stands_on_a_row_the_browser_drew() {
        for height in [400.0f32, 600.0, 800.0, 1080.0, 2160.0] {
            let s = height / 800.0;
            // The browser, as `library` measures it: the page less the head,
            // the rail, the margins and the strip standing on the legend's
            // band.
            let body = [
                0.0,
                0.0,
                (height * 1.6 - 250.0 * s).max(1.0),
                (height - 202.0 * s - FOOT * lxb_toolkit::metrics::scale_for(height) - 148.0 * s)
                    .max(1.0),
            ];
            let (_, rows) = rows_in(body, s);
            let (columns, _, cells) = cells_in(body, s);
            for library in [1usize, 3, 7, 40, 100_000] {
                for selected in [0, library / 2, library - 1] {
                    let start = first_shown(selected, false, body, s);
                    assert!(
                        selected >= start && selected - start < rows,
                        "{height}: a list of {library} with {selected} chosen"
                    );

                    let start = first_shown(selected, true, body, s);
                    assert!(
                        start.is_multiple_of(columns),
                        "{height}: a wall starts part way along a line"
                    );
                    assert!(
                        selected >= start && selected - start < cells,
                        "{height}: a wall of {library} with {selected} chosen"
                    );
                }
            }
        }
    }

    /// The strip's two bars stand clear of each other and of the row of
    /// buttons under them, at every size the window can be.
    ///
    /// They are three rows the light walks down with Up and Down, so two that
    /// touched would be two the eye reads as one — and a bar overlapping the
    /// buttons would be a pointer press that lands on whichever of them was
    /// asked for last.
    #[test]
    fn the_strips_three_rows_stand_clear_of_each_other() {
        for (w, h) in [
            (1280.0f32, 800.0f32),
            (960.0, 600.0),
            (640.0, 400.0),
            (3840.0, 2160.0),
            (1280.0, 400.0),
        ] {
            let s = h / 800.0;
            let strip = [
                24.0 * s,
                h - 64.0 * lxb_toolkit::metrics::scale_for(h) - 148.0 * s,
                w - 48.0 * s,
                148.0 * s,
            ];
            let [where_at, how_loud] = bars_at(strip, s);
            let buttons = controls_at(strip, s);
            assert!(
                where_at[1] + where_at[3] <= how_loud[1],
                "{w}x{h}: the two bars overlap"
            );
            assert!(
                how_loud[1] + how_loud[3] <= buttons[1],
                "{w}x{h}: the loudness bar reaches the buttons"
            );
            assert!(
                buttons[1] + buttons[3] <= strip[1] + strip[3],
                "{w}x{h}: the buttons hang off the strip"
            );
            for row in [where_at, how_loud] {
                assert!(row[2] > 0.0 && row[3] > 0.0, "{w}x{h}: an empty bar");
                assert!(
                    row[0] + row[2] <= strip[0] + strip[2] + 0.001,
                    "{w}x{h}: a bar runs off the strip"
                );
            }
        }
    }

    /// **Nothing in the strip is drawn over anything else, at any size the
    /// window may be.**
    ///
    /// Walked rather than reasoned about, because the fault this is here for
    /// could not be reasoned about: the three marks were centred on the whole
    /// row while the two words were hung off its left-hand end, and nothing
    /// anywhere said the two groups had to clear each other. At 1280x800 they
    /// did; at 925x747 the first mark was drawn straight through "Repeat off",
    /// two controls in one place with one press between them. It was reported
    /// from a window that size.
    ///
    /// Every number in this strip is scaled by `s`, which comes off the
    /// window's **height**, so a window that is tall for its width asks for a
    /// row far wider than it has been given — which is why a sweep and not
    /// five sizes. The guard is windows at least half as wide as they are
    /// tall: that is every phone, handheld, tablet and monitor, and past it
    /// the strip's own shape stops being the right one.
    #[test]
    fn nothing_in_the_strip_is_drawn_over_anything_else() {
        let mut w = 640.0f32;
        while w <= 7680.0 {
            let mut h = 400.0f32;
            while h <= 4320.0 {
                if w >= h * 0.5 {
                    the_strip_holds_together(w, h);
                }
                h += 80.0;
            }
            w += 160.0;
        }
    }

    fn the_strip_holds_together(w: f32, h: f32) {
        let s = h / 800.0;
        let margin = 24.0 * s;
        let strip_h = 148.0 * s;
        let strip = [
            margin,
            h - 64.0 * lxb_toolkit::metrics::scale_for(h) - strip_h,
            w - margin * 2.0,
            strip_h,
        ];
        let row = controls_at(strip, s);
        let chips = transport_at(row, s);
        for (i, chip) in chips.iter().enumerate() {
            assert!(
                chip[0] >= row[0] - 0.01 && chip[0] + chip[2] <= row[0] + row[2] + 0.01,
                "{w}x{h}: control {i} hangs off the row"
            );
            for (j, other) in chips.iter().enumerate().skip(i + 1) {
                assert!(
                    !over(*chip, *other),
                    "{w}x{h}: control {i} is drawn over control {j}"
                );
            }
        }
        let record = now_playing_at(strip, s);
        for (name, rect) in [
            ("the position bar", bars_at(strip, s)[0]),
            ("the loudness bar", bars_at(strip, s)[1]),
            ("the buttons", row),
        ] {
            assert!(
                !over(record, rect),
                "{w}x{h}: the record is drawn over {name}"
            );
        }
    }

    /// Whether two rectangles share any pixel. The hair of slack is for the
    /// ones that are laid edge to edge on purpose.
    fn over(a: [f32; 4], b: [f32; 4]) -> bool {
        a[0] < b[0] + b[2] - 0.01
            && b[0] < a[0] + a[2] - 0.01
            && a[1] < b[1] + b[3] - 0.01
            && b[1] < a[1] + a[3] - 0.01
    }

    /// A bar draws inside the row, and the row is what is lit.
    ///
    /// The fault this is here for: the loudness bar's mark was drawn flush to
    /// the left-hand end of its row, so the highlight — which *is* the row —
    /// came up hard against it while standing clear above and below. There is
    /// at least as much air at the ends now as there is over the tallest thing
    /// either bar puts in one, at every size the window may be.
    #[test]
    fn a_bar_stands_clear_of_the_row_it_is_lit_in() {
        for (w, h) in [
            (1280.0f32, 800.0f32),
            (960.0, 600.0),
            (640.0, 400.0),
            (3840.0, 2160.0),
            (1280.0, 400.0),
        ] {
            let s = h / 800.0;
            let strip = [
                24.0 * s,
                h - 64.0 * lxb_toolkit::metrics::scale_for(h) - 148.0 * s,
                w - 48.0 * s,
                148.0 * s,
            ];
            for row in bars_at(strip, s) {
                let inside = bar_inside(row, s);
                let left = inside[0] - row[0];
                let right = (row[0] + row[2]) - (inside[0] + inside[2]);
                assert!(left > 0.0, "{w}x{h}: a bar is flush to the left of its row");
                assert!(
                    right > 0.0,
                    "{w}x{h}: a bar is flush to the right of its row"
                );
                assert!(
                    (left - right).abs() < 0.001,
                    "{w}x{h}: a bar is off centre in its own row"
                );
                // The loudness mark is the tallest thing either bar holds, so
                // the air over it is the air to beat.
                let over = (row[3] - row[3].min(20.0 * s)) * 0.5;
                assert!(
                    left >= over,
                    "{w}x{h}: less air at the end of a bar than above it"
                );
                assert!(inside[2] > 0.0, "{w}x{h}: nothing left to draw in");
            }
        }
    }
}
