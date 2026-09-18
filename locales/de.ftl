# Musik — Deutsch.

app-name = Musik
app-tagline = Ihre Musik, vom Sofa aus.
preview = VORSCHAU · eine erfundene Sammlung

library = IHRE SAMMLUNG
songs = Titel
albums = Alben
artists = Interpreten
favourites = Favoriten
queue = Warteschlange

songs-subtitle = Alles, was Sie haben.
albums-subtitle = Eine Platte auflegen und bleiben.
artists-subtitle = Eine vertraute Stimme finden.
favourites-subtitle = Die Titel, zu denen Sie zurückkehren.
queue-subtitle = Was als Nächstes läuft, der Reihe nach.
album-subtitle = { $artist }
artist-subtitle = Alles von diesem Interpreten.

track = TITEL
length = DAUER
current = Läuft
song-count = { $count ->
    [one] { $count } Titel
   *[other] { $count } Titel
    }
album-count = { $count ->
    [one] { $count } Album
   *[other] { $count } Alben
    }
position-in-list = { $at } von { $of }

empty = Hier ist noch keine Musik
empty-note = Fügen Sie den Ordner mit Ihrer Musik hinzu, dann wird von dort gelesen.
empty-filter = Unter dieser Überschrift ist noch nichts.
empty-queue = Nichts in der Warteschlange
empty-queue-note = Spielen Sie einen Titel ab oder fügen Sie einen über das Menü hinzu.
scanning = Ihre Musik wird gelesen…
scanning-note = Titel erscheinen, sobald sie gefunden werden. Sie können schon vorher abspielen.

now-playing = LÄUFT GERADE
up-next = ALS NÄCHSTES
ready = Bereit, wenn Sie es sind
choose-song = Wählen Sie etwas aus Ihrer Sammlung.
nothing-queued = Nichts nach diesem Titel.
of-album = aus { $album }

options = Optionen
back = Zurück
close = Schließen
open = Öffnen
play = Abspielen
pause = Pause
add-queue = Zur Warteschlange hinzufügen
favourite = Zu Favoriten hinzufügen
unfavourite = Aus Favoriten entfernen
remove-queue = Aus der Warteschlange nehmen
clear-queue = Warteschlange leeren
add-folder = Musikordner hinzufügen
add-folder-short = Ordner hinzufügen
refresh = Sammlung neu einlesen
open-now-playing = Jetzt läuft

previous = Zurück
next = Weiter
shuffle-on = Zufall an
shuffle-off = Zufall aus
repeat-off = Keine Wiederholung
repeat-all = Alles wiederholen
repeat-one = Titel wiederholen
mute = Stumm schalten
unmute = Ton einschalten

## What order a shelf is listed in

sort-by = Sortieren nach
order-name = Name
order-name-backwards = Name rückwärts
order-artist = Interpret
order-album = Album
order-newest = Neueste zuerst
order-oldest = Älteste zuerst

unknown-artist = Unbekannter Interpret
unknown-album = Unbekanntes Album
error-save = Ihre Einstellungen konnten nicht gespeichert werden: { $error }
error-ffprobe = FFmpeg ist nicht installiert, daher kann Ihre Musik nicht gelesen werden: { $error }
error-read = { $name } konnte nicht gelesen werden
error-no-audio = In { $name } ist kein Ton
error-device = Es kann nichts abgespielt werden: { $error }
error-open = { $name } lässt sich nicht abspielen: { $error }
error-seek = In diesem Titel kann nicht gesprungen werden: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (und { $count ->
    [one] { $count } weiterer
   *[other] { $count } weitere
    })
