# Music — English (UK).
#
# This is the whole catalog and the fallback every other language falls back
# to. A message missing from a translation is answered from here; a message
# missing from here is answered with its own identifier, which is a hole
# somebody can see and search for.
#
# en-US.ftl beside this file is an overlay of the handful of words America
# writes differently, and nothing else.

## The application itself

app-name = Music
app-tagline = Your music, from across the room.
preview = PREVIEW · a made-up library

## The rail down the left-hand side

library = YOUR LIBRARY
songs = Songs
albums = Albums
artists = Artists
favourites = Favourites
queue = Queue

songs-subtitle = Everything you have.
albums-subtitle = Pick a record. Settle in.
artists-subtitle = Find a familiar voice.
favourites-subtitle = The songs you come back to.
queue-subtitle = What is going to play, in order.
album-subtitle = { $artist }
artist-subtitle = Everything by this artist.

## The browser

track = TRACK
length = TIME
current = Playing
song-count = { $count ->
    [one] { $count } song
   *[other] { $count } songs
    }
album-count = { $count ->
    [one] { $count } album
   *[other] { $count } albums
    }
position-in-list = { $at } of { $of }

empty = No music here yet
empty-note = Add the folder your music is in and it will be read from there.
empty-filter = Nothing under this heading yet.
empty-queue = Nothing is queued
empty-queue-note = Play a song, or add one to the queue from the Options menu.
scanning = Reading your music…
scanning-note = Songs appear as they are found. You can start playing before it has finished.

## The player

now-playing = NOW PLAYING
up-next = UP NEXT
ready = Ready when you are
choose-song = Choose something from your library.
nothing-queued = Nothing after this one.
of-album = from { $album }

## What the buttons do, and what is on the menu

options = Options
back = Back
close = Close
open = Open
play = Play
pause = Pause
add-queue = Add to the queue
favourite = Add to favourites
unfavourite = Take out of favourites
remove-queue = Take out of the queue
clear-queue = Empty the queue
add-folder = Add a music folder
add-folder-short = Add a folder
refresh = Read the library again
open-now-playing = Now playing

## The transport

previous = Previous
next = Next
shuffle-on = Shuffle on
shuffle-off = Shuffle off
repeat-off = Repeat off
repeat-all = Repeat all
repeat-one = Repeat one
mute = Mute
unmute = Unmute

## What order a shelf is listed in

sort-by = Sort by
order-name = Name
order-name-backwards = Name backwards
order-artist = Artist
order-album = Album
order-newest = Newest first
order-oldest = Oldest first

## When a file, a folder or the sound card will not answer

unknown-artist = Unknown artist
unknown-album = Unknown album
error-save = Your settings could not be saved: { $error }
error-ffprobe = FFmpeg is not installed, so your music cannot be read: { $error }
error-read = { $name } could not be read
error-no-audio = There is no sound in { $name }
error-device = Nothing can be played: { $error }
error-open = { $name } will not play: { $error }
error-seek = This song cannot be moved through: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (and { $count ->
    [one] { $count } other
   *[other] { $count } others
    })
