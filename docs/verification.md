# What was actually checked

This file says what has been run and what has not. Everything below was done on
this machine, against the installed **lxb-toolkit 0.9.0** at
`/usr/share/lxb-toolkit/crates`.

## Run and passing

* `cargo build --release --locked`
* `cargo test --locked` — **55 tests**. They cover the queue (end, repeat-all,
  repeat-one, shuffle visiting each occurrence once, Previous retracing what was
  played, removal keeping the current occurrence), controller navigation on both
  screens and its bounds, browsing without replacing the playing queue, album
  grouping by artist *and* title, the favourites filter, duplicate queue
  entries, stale reports from the sound worker, folder deduplication and symlink
  cycles, the page crossing in both directions, the entry stagger, the Now
  Playing page's geometry at five window sizes, the control numbering, the
  three catalog tests, the two the foot of the page gained — that the band the
  button hints sit in really holds them clear of the strip at every size the
  window may be, and that a card's rounding and a chip's are different numbers
  at every one of those sizes, which is why the light has to be told which
  shape it is — and one the light gained: that the row it is laid on is always
  a row the browser really drew, in a list and on a wall, for libraries from
  one song to a hundred thousand at five window sizes — and the five the strip
  gained when the two grooves became controls: that a direction along a bar
  moves the bar and never the light, that the light climbs out of the strip a
  row at a time before it lets go of it, that the Now Playing page walks the
  same three rows and still reaches the queue off the end of the *buttons*,
  that muting comes back at the loudness it silenced, and that the three rows
  stand clear of each other at every window size. Every one of the seven is
  also asked what pressing it does, in case a name is ever left pointing at a
  message no catalog carries.

  Five more came with the Options menu and the record at the head of the strip:
  that the record is walked to and opens the Now Playing page, that the page it
  opens has no such row of its own, that every shelf whose order is the user's
  is really put in it while the queue and an album keep their own, that the
  light stays on the song across a change of order, and — with an exhaustive
  match, so that adding one would not compile — that no row of the menu closes
  the application.

  One more when the rail's Options row went: that no row of the rail leaves the
  legend offering the menu, asked of `draw::hints` on every row of it. The menu
  is raised by the button the legend names and nothing else, and a rail row
  that raised it too was a second way to press a button already on screen.

  Two more when sorting became a row of its own: that no order is left loose in
  the Options menu and that the row named for sorting is the one that opens
  them, and that neither menu has to be scrolled in the window this application
  opens at. The second is asked of `menu::rows_of_that_fit`, which is the same
  sum the panel sizes its own window with — the menu it replaced came to twelve
  rows on the Songs shelf where ten fit.

  The navigation ones were rewritten with the model: that Down into the strip
  lands on the same row every time whatever the light was on there last, that
  Up off the top of the strip undoes the Down that got there, that the rail is
  seven rows and standing on one of its two buttons does not change which shelf
  is open, and that Right off the last column of anything goes nowhere.

  Seven more came with the bars being something a hand can really move: that a
  point along a groove is where it is along it and that either end of the row
  is the end of the bar; that the stick's push is squared so the near half of
  its travel is the slow half, never goes backwards, and reaches full speed
  only at the stop; that the bar under the light is the one a push moves and no
  other, and nothing at all is moved while the light is on a button or a shelf;
  that a drive is what the song *shows* until it is let go of, that presses
  inside one report of where the sound got to accumulate rather than each
  starting from the same stale number, and that a drive nobody is holding is
  let go of; that reaching for one bar finishes with the other, so a loudness
  somebody left behind still reaches the settings file; and that both bars
  stand clear of the row they are lit in, by at least the air there is above
  them, at every size the window may be.
* `cargo clippy --locked --all-targets -- -D warnings`
* `cargo fmt --check`
* `packaging/check.sh` — version agreement across `VERSION`, `Cargo.toml`,
  `Cargo.lock`, the spec and the AppStream release list; the toolkit requirement
  in every recipe; the three names this application answers to; the staged
  payload; `desktop-file-validate`; `appstreamcli validate`.
* `packaging/install.sh --destdir … --prefix /usr` into a scratch directory,
  and the files it stages. (There was a `Makefile` wrapping this; it is gone,
  because none of the three sibling applications had one and a second way to
  install is a second thing to keep agreeing with the first.)
* Headless renders through the real renderer at 1600×900, 1280×800, 960×600
  and 640×400:
  Songs, Albums, an album's track list, the Now Playing page, and the crossing
  caught 0.16 s in.
* **Every window size this application accepts, walked.** Two tests step
  640×400 to 7680×4320 and assert that nothing in the strip and nothing on the
  Now Playing page is drawn over anything else, hangs off its row or stands in
  the legend's band. Five sizes were not enough: the three transport marks were
  centred on the whole row while the two words were hung off its left-hand end,
  and nothing said the groups had to clear each other — at 1280×800 they did,
  and at 925×747 the first mark went straight through "Repeat off". Both tests
  were checked against the old sum and fail on it.

  The guard is windows at least half as wide as they are tall, which is every
  phone, handheld, tablet and monitor. Past that the strip's own shape — a
  record, two bars and a row of five, all side by side — stops being the right
  one, and no amount of squeezing makes it right.

  The five preview pages are **pixel for pixel identical** before and after,
  which is the other half of it: a fix for a narrow window that moves an
  ordinary one has changed something it was not asked to.
* The foot of the window at 640×400, 960×600, 1280×800, 1920×1080 and
  3840×2160, which is where the hints and the strip used to meet. The band
  holds at all five.
* The light on each of the five things that can wear one — a row of the list, a
  tab on the rail, a card on the wall of records, a chip of the transport and,
  now, a bar and the record at the head of the strip — by forcing each zone and
  each place in the strip in a scratch build and photographing it. The four that are rows wear the card's own
  corner; the chip is unchanged. The legend was read off the same pictures, to
  see that every place the light can stand names what South does there.
* **The light half way across.** A `--shot` renders two frames at one instant,
  so the second has no time in it and the glide cannot move: seeding the light
  on one thing and then pressing a direction leaves it drawn at the old
  rectangle while the new one is chosen, which is the widest a glide ever gets.
  Driven that way through all five journeys the light can make — row to row,
  record to record, tab to tab, chip to chip, and the browser out to the strip
  — to see that nothing it passes over is swallowed. That is how the covers
  being eaten by the highlight was found, and how the fix was checked.
* **The whole map, read as one.** A throwaway test walked every direction from
  every zone on both screens and printed where the light went, so that the
  model could be read rather than reasoned about. That is what said the strip
  was being handed the light at whatever row it was last on, that the buttons
  at the foot of the rail could not be reached at all, and that Right off the
  last record on the wall fell into the transport.
* The light on each of the rail's last two rows — the Queue shelf and Add a
  folder — photographed, with the legend under them: the button lights as a
  chip and the shelf keeps its own card while the light stands on it. Taken
  again when the Options row went and Add a folder came down into its place,
  because the highlight and the chip are two readings of one sum and a row
  that moves is where they would drift apart.
* **The order the shelves come out in**, photographed at three of the six with
  a scratch build that sets one before the first frame: by artist, newest
  first, and backwards. The pictures also show the listing scrolled to keep the
  light on the song it was on, which is the half of a change of order that no
  test reads.
* **What a click lands on.** `Ui::at` is asked, against a real laid-out frame,
  what a press at the middle of each of the strip's rows would hit: the
  position bar answers `POSITION`, the loudness bar `VOLUME`, the two chips and
  the three marks their own numbers, and a song in the browser its row.

  **That was not enough, and the gap is worth recording.** It proved a press
  would *arrive* and never asked what value it produced — and the value was
  wrong every single time. The pointer was being read from `Page::cursor`,
  which is the layout's own cursor and not the pointer at all: it stands at the
  left-hand edge of the page, so every press anywhere on either bar answered
  the same number and this clamped it to nought. A click on the position bar
  sent the song back to its beginning and a click on the loudness bar silenced
  it. A test that a control is reachable is not a test that it works.
* Every settled page, before and after that reordering, compared pixel for
  pixel: the library and the Now Playing page are identical and the wall of
  records differs by two parts in 255 along the top edge of the strip, which is
  the highlight's glow now falling on the strip's glass instead of under it.
* The same pages under `LC_ALL=zh_CN.UTF-8` and `LC_ALL=ru_RU.UTF-8`, to see
  that nothing is cut to an ellipsis in a language whose characters are a whole
  em wide.
* The application icon rasterised at 27, 48 and 128 pixels, which is where a
  glyph either reads or turns to mush.
* **The bars against their own highlight**, photographed with the light forced
  onto each in turn by a scratch build. The loudness mark used to come up hard
  against the left-hand edge of the highlight while standing clear of it above
  and below; it stands inside it now, by the same number a row of the list
  keeps its sleeve from the edge of its row. Every settled page compared pixel
  for pixel before and after: the Now Playing page differs by 208 parts in a
  million at two parts in 255, which is that inset and nothing else.

## The bars, driven by a real hand

The three ways a bar can be moved were each driven against the real binary on a
private rootful Xwayland (`Xwayland :7 -geometry 1400x900`) and photographed
with `import -window`. Nothing touched the user's own session and nothing was
played.

* **A pointer, through XTEST.** A click three quarters along the loudness
  groove put it at 94%; a press at its foot followed by interpolated motions to
  the right walked the handle across in three photographs and left it where the
  button came up. The drag goes on tracking when the pointer leaves the row
  vertically, which is what `Page::dragging` keeping hold of the control it
  began on means. A click on the position bar was photographed landing the
  light on the position bar, which is the same branch — the zone and the row
  are only written inside it.
* **A stick, through a uinput pad.** `ABS_X` *and* `ABS_Y` plus a button, or
  GilRs enumerates the device and never reads it; see the shell repository's
  `pad-guard-verification`. Four seconds at 0.40 moved the loudness a third of
  the way and one second at the stop moved it nine tenths, against the 0.31 and
  0.90 the rate says.
* **And that the toolkit's own repeats are not counted twice.** At 0.60 — just
  past `input::STICK_ENGAGE`, so the toolkit is making Left and Right out of
  the same push — three seconds moved the loudness to 76%, which is the smooth
  drive alone. Counted on top, twenty-nine repeats of five percent each would
  have had it at its stop.
* **The song's position, read rather than photographed**, because `--demo` pins
  what the groove draws so that a preview picture is the same picture every
  time. A scratch build printing `Music::position` every frame was driven by
  the same pad: 0.30 on the stick took the song 1.42 seconds per second and the
  stop took it 45.0, against the 1.40 and 45.0 the rate says — and letting go
  handed the groove back to the sound worker's own report.

## The media test asks something of the machine

`library::media_tests` writes tagged FLAC, MP3, M4A, M4B, MKA, WAV, Ogg, AIFF
and raw AAC fixtures with FFmpeg, reads each back through `probe`, and pulls
samples out of the Rodio decoder to prove the file really decodes. It opens **no
sound device**. A machine with no `ffmpeg` on `PATH` fails it, which is correct
— that machine cannot run this application either.

The list `library::supported` accepts was settled the same way rather than
guessed. Opus, WMA and WavPack fixtures were encoded and handed to the decoder,
which refused all three, so none of them is on it; `m4b` and `mka` were accepted
and are. APE and Musepack have no encoder here to make a fixture with and are
left off on Symphonia's own account.

## Driven as a real window

The five things reported after the first release were found and fixed by running
the real binary on a **private rootful Xwayland** (`Xwayland :7 -geometry
1400x900`), driving it with XTEST through python-xlib and photographing the
window with `import -window`. Nothing touched the user's own session. Pictures
and Videos were run on the same server for comparison, which is what said that
the folder question was the thing that differed: they ask the desktop and this
was forcing the toolkit's built-in picker.

The built-in picker was then exercised on its own with `LXB_FILE_PORTAL=0`, the
toolkit's own switch for it, to check the fallback still answers where there is
no portal. The portal path itself was **not** driven here, because raising it
would have put a chooser on the user's real desktop.

## The Options menu is read rather than photographed

`--shot` renders two frames at one instant and raises a menu on the second of
them, so the menu is open but has not been given a frame to arrive in: there is
no picture of it to take. What it says is asked of `Music::menu_rows` instead,
which is the same list `open_menu` hands to the toolkit and is split out for
exactly this reason.

**Both menus were photographed all the same**, on the private rootful Xwayland
above: a right click raised Options, which came out seven rows with no arrow at
its foot — a right click, because a right click and Y are now the only ways to
raise it; Enter on Sort by raised the orders, out of the same anchor and titled
for themselves, with Name ticked; and four presses down and Enter put the shelf
in newest-first order with the light still on the song it was on. A second menu
raised from inside `Page::chose` is accepted because the toolkit closes the
first one on the frame the row is taken, so `menu_marked` no longer refuses.

## Not exercised here

* **A physical controller.** The navigation is tested through `Music::navigate`
  rather than through a pad, and the stick was driven through a pad this
  machine made rather than one somebody is holding. This box has no evdev
  gamepad at all — see the shell repository's `pad-guard-verification` and
  `steam-controller-2-no-gamepad` — which is why `--controllers` exists, and
  why it is the first thing to ask before believing a report that the
  controller support does nothing.
* **A real sound device.** Nothing in the test suite opens one, and this session
  has no speaker. Playback, seeking, the end-of-track advance and the volume
  have been read rather than heard.
* **A long listen.** Memory over hours, and a library of tens of thousands of
  songs, are unmeasured.
* **A package build.** `packaging/check.sh` passes; `build.sh arch`,
  `debian`, `fedora` and `nix` each want their own distribution and have not
  been run.
* **Installation into a real session**, and therefore the shell's Music shelf
  actually opening this.

## About the preview pictures

`docs/*.png` are `--demo` renders at the Indigo accent, except `options.png`
and `sorting.png`, which are captures of the real binary on a private rootful
Xwayland — a menu cannot be photographed with `--shot` here, because it is
raised on the last of two frames at one instant and never given one to arrive
in. Nothing in either route touches the user's own session.

The commands that make each one are in [`design.md`](design.md), under *Where
the pictures come from*, along with why the accent is pinned.

`--demo` pins the song's position at 68 seconds, so that a preview picture is
the same picture every time. That is the one thing about a seek a picture of
this application cannot be used to check.

## Known, and left alone

The toolkit's built-in file picker draws as a very transparent panel with the
page reading through it. That is the component's own look — it is the same in
every application that falls back to it — and changing it means changing
`lxb-render`, which is a release of the whole design language rather than a
change to this program. It is only ever seen where there is no desktop portal.

The 640×400 rough edge is gone: the transport is marks now, and a mark needs no
label width, so the smallest window this accepts is as clean as the largest.
