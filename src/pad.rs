//! The one analog control the interface language has no word for.
//!
//! `lxb-input` reads every controller on the machine and answers in *actions*
//! — Left, Accept, Menu, Next — which is deliberate: it is what makes a pad, a
//! keyboard and a wheel one interface instead of three, and an application
//! that went behind it would be inventing a second set of controls nobody else
//! on this desktop has. So nearly all of this application's input comes from
//! there and nothing here duplicates it.
//!
//! The stick is the exception, and only because a bar is the exception. An
//! action is a thing that happened; a stick is a quantity — *how far* — and
//! there is no honest way to say "a third of the way over" in a list of
//! actions. The toolkit turns the stick into Left and Right at a fixed
//! cadence, which is exactly right for walking a list and exactly wrong for
//! setting a bar: a thumb held a little off centre is asking to creep along
//! the song, not to jump ten seconds again.
//!
//! So this opens its own reader for exactly that, and answers one number. It
//! maps nothing, decides no cadence, applies no dead zone and has no opinion
//! about any button: `lxb_toolkit::input` remains the only thing in this
//! application that says what a control *means*, and what the number means is
//! decided in `app.rs` beside the speeds it drives. Two readers on one device
//! is not a conflict — a controller is not a Wayland input device and never
//! passes through a compositor, so every program on the machine already reads
//! the same pad at the same time.
//!
//! The film player and the photo viewer each keep one of these for their own
//! analog control; this is the third, and the first to read a stick.

use gilrs::{Axis, Gilrs, GilrsBuilder};

pub struct Pad {
    pads: Option<Gilrs>,
    /// Why there is no reader, if there is none. Said once, at startup.
    trouble: Option<String>,
}

/// One controller, as it is seen — for `--controllers`, which exists because
/// "the controller does nothing" has two completely different causes and no
/// way to tell them apart from the outside.
pub struct Found {
    pub name: String,
    pub mapped: bool,
    pub stick: bool,
}

impl Default for Pad {
    fn default() -> Self {
        Self::new()
    }
}

impl Pad {
    pub fn new() -> Pad {
        // Force feedback off, as `lxb-input` does. A rumble request nothing
        // answers blocks for thirty seconds, and this reader is opened on the
        // way to the first frame.
        let (pads, trouble) = match GilrsBuilder::new().with_force_feedback(false).build() {
            Ok(pads) => (Some(pads), None),
            Err(err) => (None, Some(err.to_string())),
        };
        Pad { pads, trouble }
    }

    pub fn trouble(&self) -> Option<&str> {
        self.trouble.as_deref()
    }

    /// Pump the queue, once a frame.
    ///
    /// Not for the events — those are `lxb-input`'s business — but because a
    /// gamepad's stored axis values are only brought up to date by draining
    /// them. Without this the stick reads whatever it was when the device was
    /// opened, for ever.
    pub fn settle(&mut self) {
        if let Some(pads) = self.pads.as_mut() {
            while pads.next_event().is_some() {}
        }
    }

    /// How far the stick is pushed sideways: −1 at the left stop, 1 at the
    /// right, nought at rest.
    ///
    /// **Raw on purpose.** The dead zone and the curve belong to whatever is
    /// being driven, and this number has to stay comparable with
    /// `lxb_toolkit::input::STICK_ENGAGE` — because the toolkit is making its
    /// own Left and Right repeats out of the very same stick, and something
    /// has to be able to tell that those repeats and this push are one hand.
    ///
    /// The left stick, because that is the one the toolkit steers with.
    /// Whichever pad is pushed furthest wins, so it does not matter which of
    /// several is in hand.
    pub fn push(&self) -> f32 {
        let Some(pads) = self.pads.as_ref() else {
            return 0.0;
        };
        let mut most = 0.0_f32;
        for (_, pad) in pads.gamepads() {
            let value = pad.value(Axis::LeftStickX);
            if !value.is_finite() {
                continue;
            }
            let value = value.clamp(-1.0, 1.0);
            if value.abs() > most.abs() {
                most = value;
            }
        }
        most
    }

    pub fn found(&self) -> Vec<Found> {
        let Some(pads) = self.pads.as_ref() else {
            return Vec::new();
        };
        pads.gamepads()
            .map(|(_, pad)| Found {
                name: pad.name().to_string(),
                mapped: pad.mapping_source() == gilrs::MappingSource::SdlMappings,
                // What the pad *has*, not what it has sent: `axis_data` is
                // the last reading and there is none until somebody touches
                // it, so asking that would tell everybody their stick was
                // missing right up until they used it.
                stick: pad.axis_code(Axis::LeftStickX).is_some(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reader_that_found_nothing_is_not_a_failure() {
        // Never panics and never blocks, whatever this machine has plugged in
        // — including nothing at all, which is the ordinary case on a desktop.
        let pad = Pad::new();
        assert_eq!(pad.push(), 0.0, "a stick nobody is touching is not pushed");
        // And it can be asked what it found without opening anything.
        let _ = pad.found();
    }
}
