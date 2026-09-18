//! The Now Playing page: one record, as large as the window will allow.
//!
//! It is laid out inside a rectangle rather than inside the window, because
//! during a crossing the page is drawn moved — grown out of the card that was
//! pressed and clipped to it. Everything here is therefore relative to what it
//! is handed and nothing reads the window directly.
use crate::{
    app::{entry, Card, Music, Zone},
    draw::{self, fit, sleeve, Paint, Placed},
    i18n,
    library::time,
    message,
};
use lxb_render::{Align, Selection, Ui};
use lxb_toolkit::{material::Surface, motion, palette::Role, typography::Text};

/// How much of the page the up-next column down the right-hand side takes.
const QUEUE_SHARE: f32 = 0.30;

/// The one place every part of the page is measured from.
///
/// The two bars and the transport run the **whole** width of the page, under
/// the up-next column rather than beside it. Five controls sharing two thirds
/// of a window are five controls whose words are all cut to an ellipsis, and a
/// button whose label has been cut is a button nobody can read.
struct Shape {
    margin: f32,
    art: [f32; 4],
    words: [f32; 4],
    queue: [f32; 4],
    /// The two bars the light can stand on: how far through the song, and how
    /// loud. Rows rather than grooves, for the reason `draw::bars_at` gives.
    bars: [[f32; 4]; 2],
    controls: [f32; 4],
}

fn shape(at: [f32; 4], s: f32, foot: f32) -> Shape {
    let [x, y, w, h] = at;
    let margin = 44.0 * s;
    let top = y + 100.0 * s;
    let across = w - margin * 2.0;
    let row_h = 46.0 * s;
    // `foot` is the band the button legend has, handed in rather than worked
    // out here because it is the toolkit's number and this file measures in
    // its own — see `draw::foot`. The sheet stops where the band starts, and
    // the transport is the last thing on the sheet.
    let controls_y = y + h - foot - 26.0 * s - row_h;
    let bar_h = 30.0 * s;
    // The loudness bar sits directly under the position bar, the same stack
    // the strip under the library has, so that the light walks the same way on
    // both screens.
    let loud_y = controls_y - 12.0 * s - bar_h;
    let where_y = loud_y - 4.0 * s - bar_h;
    let body_bottom = where_y - 24.0 * s;
    let queue_w = (w * QUEUE_SHARE).min(380.0 * s);
    let left_w = across - queue_w - 28.0 * s;
    let side = (body_bottom - top)
        .min(left_w * 0.56)
        .clamp(48.0 * s, 620.0 * s);
    // Centred in the room it has rather than hung from the top: a square in a
    // rectangle leaves a band somewhere, and a band under a record is a page
    // that looks as though something has failed to load.
    let art = [
        x + margin,
        top + (body_bottom - top - side) * 0.5,
        side,
        side,
    ];
    let words_x = art[0] + art[2] + 28.0 * s;
    Shape {
        margin,
        art,
        words: [words_x, art[1], x + margin + left_w - words_x, side],
        queue: [
            x + w - margin - queue_w,
            y + 74.0 * s,
            queue_w,
            body_bottom - (y + 74.0 * s),
        ],
        bars: [
            [x + margin, where_y, across, bar_h],
            // Narrower, and centred: a volume groove the width of the window
            // is a control that wants a hand steadier than anybody has.
            [
                x + margin + (across - (across * 0.34).clamp(200.0 * s, 420.0 * s)) * 0.5,
                loud_y,
                (across * 0.34).clamp(200.0 * s, 420.0 * s).min(across),
                bar_h,
            ],
        ],
        controls: [x + margin, controls_y, across, row_h],
    }
}

/// Where the large sleeve goes on a page of this size.
///
/// Public because a crossing has to know it before the page is drawn: the
/// sleeve grows out of the card's own picture into exactly this rectangle, and
/// a stand-in that landed anywhere else would jump on the last frame.
pub fn sleeve_rect(at: [f32; 4], s: f32, foot: f32) -> [f32; 4] {
    shape(at, s, foot).art
}

/// Draw the page. Answers where it put the up-next rows, so a press can name
/// one.
///
/// `live` is whether this is the page that answers presses — during a crossing
/// neither page is, because both would write their targets down over each
/// other. `with_sleeve` is whether it draws its own record sleeve or whether a
/// crossing is growing one into that very place.
#[allow(clippy::too_many_arguments)]
pub fn page(
    state: &Music,
    ui: &mut Ui,
    light: &mut Selection,
    at: [f32; 4],
    paint: Paint,
    with_sleeve: bool,
    placed: &mut Placed,
) {
    let (icons, s, live) = (paint.icons, paint.s, paint.live);
    let [x, y, w, h] = at;
    let foot = draw::foot(ui);
    let shape = shape(at, s, foot);
    let since = state.seconds - state.arrived;

    // The page's own glass. One sheet rather than several cards: this is a
    // page, and a page is a sheet.
    ui.card(
        [
            x + shape.margin - 20.0 * s,
            y + 56.0 * s,
            w - shape.margin * 2.0 + 40.0 * s,
            h - 56.0 * s - foot,
        ],
        Surface::Panel,
        Role::Glass,
        0.62,
    );

    // On the sheet, and under everything that stands on it — see `light_on`.
    if let Some((rect, lit)) = light_on(state, &shape, paint, since) {
        draw::focus(ui, light, rect, lit);
    }

    label(
        ui,
        [x + shape.margin, y + 74.0 * s, shape.words[2], 24.0 * s],
        Text::Caption,
        i18n::text("now-playing"),
        Role::TextSoft,
    );

    if with_sleeve {
        let fade = ((state.seconds - state.sleeve_at) / lxb_toolkit::motion::duration::COLOUR_FADE)
            .clamp(0.0, 1.0);
        sleeve(
            ui,
            shape.art,
            state.sleeve(),
            icons,
            motion::smoothstep(fade),
        );
    }
    // A record breathing under the sleeve while it plays: the one thing on
    // this page that is not still. It is a light rather than a picture of the
    // sound, so it says "this is going" without pretending to be a reading of
    // anything.
    if state.audio.status.playing {
        let breath = 0.10 + 0.16 * motion::pulse(state.seconds);
        let out = shape.art[2] * 0.06;
        ui.glow(
            [
                shape.art[0] - out,
                shape.art[1] - out,
                shape.art[2] + out * 2.0,
                shape.art[3] + out * 2.0,
            ],
            Role::Accent,
            breath,
        );
    }

    let [words_x, _, words_w, _] = shape.words;
    let current = state.current();
    let arrived = entry(since, 0);
    let lift = (1.0 - arrived) * 20.0 * s;
    let middle = shape.art[1] + shape.art[3] * 0.5;
    label_faded(
        ui,
        [words_x, middle - 78.0 * s + lift, words_w, 46.0 * s],
        Text::Title,
        current
            .map(|track| track.title.as_str())
            .unwrap_or(i18n::text("ready")),
        Role::Text,
        arrived,
    );
    label_faded(
        ui,
        [words_x, middle - 24.0 * s + lift, words_w, 34.0 * s],
        Text::Body,
        current
            .map(|track| track.artist.as_str())
            .unwrap_or(i18n::text("choose-song")),
        Role::TextSoft,
        entry(since, 1),
    );
    if let Some(album) = current.map(|track| track.album.clone()) {
        let line = message!("of-album", "album" => album);
        label_faded(
            ui,
            [words_x, middle + 18.0 * s + lift, words_w, 28.0 * s],
            Text::Caption,
            &line,
            Role::TextSoft,
            entry(since, 2),
        );
    }
    // Shuffle and repeat are said in words here as well as pictured on the
    // row, because on this page there is room and a state nobody can see is a
    // state nobody trusts.
    let standing = format!(
        "{}  ·  {}",
        draw::transport_label(state, crate::app::FIRST_BUTTON),
        draw::transport_label(state, crate::app::FIRST_BUTTON + 1),
    );
    label_faded(
        ui,
        [words_x, middle + 56.0 * s + lift, words_w, 26.0 * s],
        Text::Caption,
        &standing,
        Role::TextSoft,
        entry(since, 3),
    );

    let length = current.map(|track| track.duration).unwrap_or(0.0);
    let position = if state.demo { 68.0 } else { state.position() };
    let where_at = draw::position_bar(ui, shape.bars[0], position, length, paint);
    let loud = draw::volume(state, ui, shape.bars[1], paint);
    draw::controls(state, ui, shape.controls, paint);

    let queued = up_next(state, ui, shape.queue, paint, since);
    if live {
        ui.spot(draw::POSITION, shape.bars[0]);
        placed.queued = queued;
        placed.volume = loud;
        placed.position = where_at;
    }
}

/// What the queue column is showing: the entries below the one playing, how
/// many of them fit, and which is the first one drawn.
///
/// Read by [`up_next`] as it draws them and by [`light_on`] before the page is
/// drawn at all.
fn queued_in(state: &Music, at: [f32; 4], s: f32) -> (Vec<(usize, usize)>, usize, usize) {
    let row_h = 58.0 * s;
    let shown = (((at[3] - 32.0 * s) / row_h).floor().max(1.0) as usize).min(12);
    let from = state.queue.position.map(|at| at + 1).unwrap_or(0);
    let entries: Vec<(usize, usize)> = state
        .queue
        .entries
        .iter()
        .enumerate()
        .skip(from)
        .map(|(at, track)| (at, *track))
        .collect();
    let start = state
        .queued
        .saturating_sub(from)
        .saturating_sub(shown.saturating_sub(1))
        .min(entries.len().saturating_sub(1));
    (entries, shown, start)
}

/// Where a row of the queue goes.
fn row_at(at: [f32; 4], s: f32, since: f32, slot: usize, dip: Option<f32>) -> [f32; 4] {
    let [x, y, w, _] = at;
    let row_h = 58.0 * s;
    let lift = (1.0 - entry(since, slot + 2)) * 18.0 * s;
    motion::pressed(
        [
            x,
            y + 32.0 * s + slot as f32 * row_h + lift,
            w,
            row_h - 6.0 * s,
        ],
        dip,
    )
}

/// Where the one light on this page stands, and what shape it is.
///
/// The Now Playing page's half of the rule `draw::light_on` keeps for the
/// library: one `Selection` is handed between the transport and the queue, so
/// it is laid down before either of them is drawn rather than by whichever of
/// them happens to own it. A light drawn by the queue is a light drawn over
/// the record, the words beside it and the whole transport row.
fn light_on(
    state: &Music,
    shape: &Shape,
    paint: Paint,
    since: f32,
) -> Option<([f32; 4], draw::Lit)> {
    if !paint.live {
        return None;
    }
    let s = paint.s;
    let dip = draw::press(state, true).through();
    match state.zone {
        // This page has no record row of its own — it *is* the Now Playing
        // page — so the two bars are the whole of what is not a button here,
        // and they are numbered one and two.
        Zone::Transport if state.transport < crate::app::FIRST_BUTTON => Some((
            shape.bars[state
                .transport
                .saturating_sub(crate::app::BAR_POSITION)
                .min(1)],
            draw::Lit::Card,
        )),
        Zone::Queue => {
            let (entries, shown, start) = queued_in(state, shape.queue, s);
            let slot = entries
                .iter()
                .skip(start)
                .take(shown)
                .position(|(at, _)| *at == state.queued)?;
            Some((row_at(shape.queue, s, since, slot, dip), draw::Lit::Card))
        }
        // Everything that is not the queue on this page is the transport, and
        // the transport is where the light goes when the page opens.
        _ => draw::transport_at(shape.controls, s)
            .get(state.transport.saturating_sub(crate::app::FIRST_BUTTON))
            .copied()
            .map(|rect| (rect, draw::Lit::Chip)),
    }
}

/// What is coming, down the right-hand side.
fn up_next(state: &Music, ui: &mut Ui, at: [f32; 4], paint: Paint, since: f32) -> Vec<Card> {
    let (icons, s, live) = (paint.icons, paint.s, paint.live);
    let [x, y, w, _] = at;
    label(
        ui,
        [x, y, w, 24.0 * s],
        Text::Caption,
        i18n::text("up-next"),
        Role::TextSoft,
    );
    let (entries, shown, start) = queued_in(state, at, s);
    if entries.is_empty() {
        label(
            ui,
            [x, y + 38.0 * s, w, 26.0 * s],
            Text::Caption,
            i18n::text("nothing-queued"),
            Role::TextSoft,
        );
        return Vec::new();
    }
    // **Only the row under the light wears anything**, the same rule the
    // library's list keeps and for the same reason. Up next had it the other
    // way about — a card on every row *but* the chosen one — which is the same
    // column of buttons read backwards. The light itself was laid down before
    // the page; see `light_on`.
    let mut cards = Vec::new();
    for (slot, (queued, track)) in entries.iter().copied().skip(start).take(shown).enumerate() {
        let Some(song) = state.tracks.get(track) else {
            continue;
        };
        let arrived = entry(since, slot + 2);
        let chosen = live && state.zone == Zone::Queue && state.queued == queued;
        let rect = row_at(at, s, since, slot, draw::press(state, chosen).through());
        let art = [rect[0] + 6.0 * s, rect[1] + 6.0 * s, 40.0 * s, 40.0 * s];
        sleeve(ui, art, song.cover.as_deref(), icons, arrived);
        let words = w - 58.0 * s - 56.0 * s;
        label_faded(
            ui,
            [rect[0] + 54.0 * s, rect[1] + 4.0 * s, words, 26.0 * s],
            Text::Label,
            &song.title,
            Role::Text,
            arrived,
        );
        label_faded(
            ui,
            [rect[0] + 54.0 * s, rect[1] + 28.0 * s, words, 22.0 * s],
            Text::Caption,
            &song.artist,
            Role::TextSoft,
            arrived,
        );
        let tint = ui.tinted(Role::TextSoft, arrived);
        ui.label_tinted(
            [
                rect[0] + w - 56.0 * s,
                rect[1] + 16.0 * s,
                50.0 * s,
                24.0 * s,
            ],
            Text::Caption,
            &time(song.duration),
            tint,
            Align::Right,
        );
        if live {
            ui.spot(draw::QUEUED + slot as u32, rect);
            cards.push(Card {
                index: queued,
                rect,
                art,
            });
        }
    }
    cards
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

#[cfg(test)]
mod tests {
    use super::*;

    /// **Nothing on the Now Playing page is drawn over anything else, at any
    /// size the window may be** — the same walk the strip under the library
    /// gets, and for the same reason. See `draw::nothing_in_the_strip_is_drawn_
    /// over_anything_else`, which is where the fault was.
    #[test]
    fn nothing_on_the_page_is_drawn_over_anything_else() {
        let mut w = 640.0f32;
        while w <= 7680.0 {
            let mut h = 400.0f32;
            while h <= 4320.0 {
                if w >= h * 0.5 {
                    the_page_holds_together(w, h);
                }
                h += 80.0;
            }
            w += 160.0;
        }
    }

    fn the_page_holds_together(w: f32, h: f32) {
        let s = h / 800.0;
        let foot = 64.0 * lxb_toolkit::metrics::scale_for(h);
        let shape = shape([0.0, 0.0, w, h], s, foot);
        let named = [
            ("the record", shape.art),
            ("the words", shape.words),
            ("up next", shape.queue),
            ("the position bar", shape.bars[0]),
            ("the loudness bar", shape.bars[1]),
            ("the buttons", shape.controls),
        ];
        for (i, (name, rect)) in named.iter().enumerate() {
            assert!(rect[2] > 0.0 && rect[3] > 0.0, "{w}x{h}: {name} is empty");
            assert!(
                rect[0] >= -0.01 && rect[0] + rect[2] <= w + 0.01,
                "{w}x{h}: {name} hangs off the page"
            );
            assert!(
                rect[1] >= -0.01 && rect[1] + rect[3] <= h - foot + 0.01,
                "{w}x{h}: {name} stands in the legend's band"
            );
            for (other, with) in named.iter().skip(i + 1) {
                assert!(!over(*rect, *with), "{w}x{h}: {name} is drawn over {other}");
            }
        }
        let chips = crate::draw::transport_at(shape.controls, s);
        for i in 0..chips.len() {
            for j in i + 1..chips.len() {
                assert!(
                    !over(chips[i], chips[j]),
                    "{w}x{h}: control {i} is drawn over control {j}"
                );
            }
        }
    }

    fn over(a: [f32; 4], b: [f32; 4]) -> bool {
        a[0] < b[0] + b[2] - 0.01
            && b[0] < a[0] + a[2] - 0.01
            && a[1] < b[1] + b[3] - 0.01
            && b[1] < a[1] + a[3] - 0.01
    }

    /// The sleeve is square, inside the page, and leaves room for the words
    /// beside it and the transport under it at every size the window can be.
    #[test]
    fn the_sleeve_always_fits_the_page_it_is_on() {
        for (w, h) in [
            (1280.0f32, 800.0f32),
            (960.0, 600.0),
            (640.0, 400.0),
            (3840.0, 2160.0),
            (1280.0, 400.0),
        ] {
            let s = h / 800.0;
            let foot = draw::FOOT * lxb_toolkit::metrics::scale_for(h);
            let art = sleeve_rect([0.0, 0.0, w, h], s, foot);
            assert!(art[2] > 0.0 && (art[2] - art[3]).abs() < 1e-3, "{w}x{h}");
            assert!(art[0] >= 0.0 && art[1] >= 0.0, "{w}x{h}");
            assert!(art[0] + art[2] <= w, "{w}x{h}: it is wider than the page");
            assert!(
                art[2] <= (w * (1.0 - QUEUE_SHARE)),
                "{w}x{h}: it takes the queue's room"
            );
        }
    }

    /// And it moves with the page, because during a crossing the page is drawn
    /// somewhere other than where the window is.
    #[test]
    fn the_page_is_laid_out_where_it_is_put() {
        let foot = draw::FOOT * lxb_toolkit::metrics::scale_for(800.0);
        let here = sleeve_rect([0.0, 0.0, 1280.0, 800.0], 1.0, foot);
        let there = sleeve_rect([100.0, 50.0, 1280.0, 800.0], 1.0, foot);
        assert_eq!(there[0] - here[0], 100.0);
        assert_eq!(there[1] - here[1], 50.0);
        assert_eq!(there[2], here[2]);
    }

    /// Every up-next row this page can draw is numbered clear of everything
    /// else the two screens number.
    #[test]
    fn the_up_next_rows_are_numbered_clear_of_everything_else() {
        const { assert!(draw::QUEUED > draw::CONTROL + crate::app::TRANSPORT as u32) };
        const { assert!(draw::QUEUED + 12 < draw::ROW) };
    }
}
