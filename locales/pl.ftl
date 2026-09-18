# Muzyka — Polski.

app-name = Muzyka
app-tagline = Twoja muzyka, z kanapy.
preview = PODGLĄD · zmyślona biblioteka

library = TWOJA BIBLIOTEKA
songs = Utwory
albums = Albumy
artists = Wykonawcy
favourites = Ulubione
queue = Kolejka

songs-subtitle = Wszystko, co masz.
albums-subtitle = Wybierz płytę. Rozgość się.
artists-subtitle = Znajdź znajomy głos.
favourites-subtitle = Utwory, do których wracasz.
queue-subtitle = To, co zaraz zagra, po kolei.
album-subtitle = { $artist }
artist-subtitle = Wszystko tego wykonawcy.

track = UTWÓR
length = CZAS
current = Gra teraz
song-count = { $count ->
    [one] { $count } utwór
    [few] { $count } utwory
    [many] { $count } utworów
   *[other] { $count } utworu
    }
album-count = { $count ->
    [one] { $count } album
    [few] { $count } albumy
    [many] { $count } albumów
   *[other] { $count } albumu
    }
position-in-list = { $at } z { $of }

empty = Nie ma tu jeszcze muzyki
empty-note = Dodaj folder, w którym jest twoja muzyka, a zostanie stamtąd wczytana.
empty-filter = Pod tym nagłówkiem jeszcze nic nie ma.
empty-queue = Kolejka jest pusta
empty-queue-note = Odtwórz utwór albo dodaj go do kolejki z menu opcji.
scanning = Wczytywanie twojej muzyki…
scanning-note = Utwory pojawiają się w miarę znajdowania. Możesz zacząć słuchać wcześniej.

now-playing = TERAZ GRA
up-next = NASTĘPNIE
ready = Gotowe, kiedy tylko zechcesz
choose-song = Wybierz coś ze swojej biblioteki.
nothing-queued = Nic po tym utworze.
of-album = z { $album }

options = Opcje
back = Wstecz
close = Zamknij
open = Otwórz
play = Odtwórz
pause = Pauza
add-queue = Dodaj do kolejki
favourite = Dodaj do ulubionych
unfavourite = Usuń z ulubionych
remove-queue = Usuń z kolejki
clear-queue = Wyczyść kolejkę
add-folder = Dodaj folder z muzyką
add-folder-short = Dodaj folder
refresh = Wczytaj bibliotekę ponownie
open-now-playing = Teraz odtwarzane

previous = Poprzedni
next = Następny
shuffle-on = Losowo włączone
shuffle-off = Losowo wyłączone
repeat-off = Bez powtarzania
repeat-all = Powtarzaj wszystko
repeat-one = Powtarzaj utwór
mute = Wycisz
unmute = Włącz dźwięk

## What order a shelf is listed in

sort-by = Sortuj według
order-name = Nazwa
order-name-backwards = Nazwa od końca
order-artist = Wykonawca
order-album = Album
order-newest = Najnowsze najpierw
order-oldest = Najstarsze najpierw

unknown-artist = Nieznany wykonawca
unknown-album = Nieznany album
error-save = Nie udało się zapisać twoich ustawień: { $error }
error-ffprobe = FFmpeg nie jest zainstalowany, więc nie można wczytać twojej muzyki: { $error }
error-read = Nie udało się odczytać { $name }
error-no-audio = W { $name } nie ma dźwięku
error-device = Nic nie da się odtworzyć: { $error }
error-open = { $name } nie chce zagrać: { $error }
error-seek = Nie można przewinąć tego utworu: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (i { $count ->
    [one] { $count } inny
    [few] { $count } inne
    [many] { $count } innych
   *[other] { $count } innego
    })
