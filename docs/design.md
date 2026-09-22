# How Songonsole is built, and why

This is the long answer. [The README](../README.md) is the short one.
[`verification.md`](verification.md) says what has been run against it.

About five and a half thousand lines over the toolkit, and fourteen hundred
more of tests.

## It does *not* draw its own window

Both of its siblings do, and the reason this one need not is a number:
`Ui::picture` reads a file into a 512-pixel cell of a shared atlas — short for
a photograph filling a 4K screen, and ample for a record sleeve. Nothing here
needs a pass of its own, so `lxb-app` owns the window, the frame loop, the
controls and the sounds, this calls one page function per frame, and every page
can be drawn over by a menu, a dialog and the file question.

## Everything that moves is a function of the scene clock

Nothing is accumulated frame by frame. That is not tidiness. `--shot` draws two
frames at one instant, so a `dt` of nought would freeze every animation at
whatever it happened to be, and a picture of an animation would be a picture of
nothing. An animation that knows when it began can be asked what it looks like
at any moment — which is also what `--after` is.

The one exception is a bar somebody is moving, which is a rate and has to
accumulate; it is kept in one place and let go of when the hand comes off.

## Scanning and sound each have a worker

Twenty thousand songs is twenty thousand `ffprobe` runs and none of them may
hold up a frame; opening a device, a file or a seek may not interrupt a press
either. Reports from the sound worker carry a generation, so a late
end-of-track event from the song before cannot advance the queue past the one
that just started.

## The stick, and nothing else

`pad.rs` is the one place this goes past `lxb-input`, and only for the stick.
An action is a thing that happened and a stick is a quantity, and there is no
honest way to say "a third of the way over" in a list of actions. The number
drives a *speed* rather than a step, squared on the way in so that the useful
half of the travel is the slow half. It maps nothing, decides no cadence and
has no opinion about any button: `lxb_toolkit::input` stays the only thing here
that decides what a control *means*.

## One transport, drawn twice

**The transport is the same eight things on both screens**, in the same order:
the record that says what is playing, the bar that says how far through the
song it is, the bar that says how loud, then shuffle and repeat, then the three
that move through the music. Drawing two transports is how two transports come
to disagree about what the middle button says. The Now Playing page has no
record row of its own — it is what that row opens — so the walk starts on the
position bar there.

Three of the five buttons are **marks** — the same `media-previous`,
`media-play`, `media-pause` and `media-next` the shell's own media card draws,
so pressing Play here and pressing Play in the guide are plainly the same act.
Shuffle and repeat are not pictured, because the design language has no shuffle
mark and no repeat mark and the nearest neighbours are both something else:
`sort` is a sorting order and `refresh` is reading a folder again. A word is
better than the wrong mark.

## The shape of the walk

The rail is **six rows** — the five shelves, then Add a folder — and standing
on the button does not change which shelf is open. The strip is **four**: the
record, the position bar, the loudness bar, and the row of buttons. What the
light lands on when it crosses between them is the row nearest where it came
from — the top of the strip coming down, and whatever handed the light over
going back up. Never the row the light happened to be on the last time it was
there, which is a down that lands somewhere different every time it is pressed.

The Now Playing page is the same model laid out the other way round — the
up-next list stands above the strip rather than beside it. Left out of the
queue is the strip as well, because a queue can run to hundreds and pressing
down through all of them to reach Play is not a way out.

**Nothing in the Options menu closes Music.** Back does, from the rail. And
nothing on the rail opens the Options menu either: it is raised by the button
the legend names and nothing else.

## Where the pictures come from

`--demo` is a made-up library, plainly labelled *PREVIEW · a made-up library*,
with no audio behind it and nothing of the user's touched — it never writes a
setting. The records have no sleeves, which is why every one of them wears the
application's own mark: that is what the program really draws for a record
nobody supplied artwork for. `--demo` also pins the song's position at 68
seconds, so that a preview picture is the same picture every time; that is the
one thing about a seek a picture of this application cannot be used to check.

The pictures are taken at the **Indigo** accent, which is not any particular
machine's. The accent is the one setting a picture of the interface cannot help
stating, and shots taken on different days in different colours would read as
different programs. Regenerate them with a scratch settings file rather than by
changing anybody's desktop:

```sh
mkdir -p /tmp/lxb-shot/lxb
printf 'accent = "Indigo"\n' > /tmp/lxb-shot/lxb/shell.toml
export XDG_CONFIG_HOME=/tmp/lxb-shot

songonsole --demo --shot docs/library.png  --width 1600 --height 900
songonsole --demo --shot docs/albums.png   --width 1600 --height 900 --view albums
songonsole --demo --shot docs/playing.png  --width 1600 --height 900 --playing
songonsole --demo --shot docs/opening.png  --width 1600 --height 900 --playing --after 0.16
songonsole --demo --shot docs/handheld.png --width  960 --height 600 --view albums
```

`--width`/`--height` and `--size WxH` are the same thing; the first pair is
what the other three applications take, so one line photographs any of them.

**`docs/options.png` and `docs/sorting.png` are the two exceptions**, because a
menu cannot be photographed with `--shot` here — it is raised on the last of
two frames at one instant and never given one to arrive in. They are captures
of the real binary on a private rootful Xwayland, at the same Indigo accent as
the rest, and nothing touched the user's own session:

```sh
WAYLAND_DISPLAY=wayland-0 Xwayland :7 -geometry 1400x900 -noreset &
XDG_CONFIG_HOME=/tmp/lxb-shot DISPLAY=:7 WAYLAND_DISPLAY= songonsole --demo &
# right-click a row for Options, then Enter on Sort by, and
# import -display :7 -window <id> docs/options.png
```

![Opening out of the row it was pressed on](opening.png)
![The Options menu](options.png)
![The orders behind Sort by](sorting.png)
