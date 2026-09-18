# Musique — Français.

app-name = Musique
app-tagline = Votre musique, depuis le canapé.
preview = APERÇU · une bibliothèque inventée

library = VOTRE BIBLIOTHÈQUE
songs = Morceaux
albums = Albums
artists = Artistes
favourites = Favoris
queue = File d'attente

songs-subtitle = Tout ce que vous avez.
albums-subtitle = Choisissez un disque. Installez-vous.
artists-subtitle = Retrouvez une voix familière.
favourites-subtitle = Les morceaux sur lesquels vous revenez.
queue-subtitle = Ce qui va être lu, dans l'ordre.
album-subtitle = { $artist }
artist-subtitle = Tout de cet artiste.

track = MORCEAU
length = DURÉE
current = En cours
song-count = { $count ->
    [one] { $count } morceau
   *[other] { $count } morceaux
    }
album-count = { $count ->
    [one] { $count } album
   *[other] { $count } albums
    }
position-in-list = { $at } sur { $of }

empty = Pas encore de musique ici
empty-note = Ajoutez le dossier où se trouve votre musique et elle sera lue depuis là.
empty-filter = Rien sous cette rubrique pour l'instant.
empty-queue = La file d'attente est vide
empty-queue-note = Lisez un morceau, ou ajoutez-en un depuis le menu Options.
scanning = Lecture de votre musique…
scanning-note = Les morceaux apparaissent au fur et à mesure. Vous pouvez lire avant la fin.

now-playing = LECTURE EN COURS
up-next = À SUIVRE
ready = Prêt quand vous l'êtes
choose-song = Choisissez quelque chose dans votre bibliothèque.
nothing-queued = Rien après celui-ci.
of-album = extrait de { $album }

options = Options
back = Retour
close = Fermer
open = Ouvrir
play = Lire
pause = Pause
add-queue = Ajouter à la file d'attente
favourite = Ajouter aux favoris
unfavourite = Retirer des favoris
remove-queue = Retirer de la file d'attente
clear-queue = Vider la file d'attente
add-folder = Ajouter un dossier de musique
add-folder-short = Ajouter un dossier
refresh = Relire la bibliothèque
open-now-playing = Lecture en cours

previous = Précédent
next = Suivant
shuffle-on = Aléatoire activé
shuffle-off = Aléatoire désactivé
repeat-off = Pas de répétition
repeat-all = Tout répéter
repeat-one = Répéter le morceau
mute = Couper le son
unmute = Rétablir le son

## What order a shelf is listed in

sort-by = Trier par
order-name = Nom
order-name-backwards = Nom à l’envers
order-artist = Artiste
order-album = Album
order-newest = Le plus récent d’abord
order-oldest = Le plus ancien d’abord

unknown-artist = Artiste inconnu
unknown-album = Album inconnu
error-save = Vos réglages n'ont pas pu être enregistrés : { $error }
error-ffprobe = FFmpeg n'est pas installé, votre musique ne peut donc pas être lue : { $error }
error-read = { $name } n'a pas pu être lu
error-no-audio = Il n'y a pas de son dans { $name }
error-device = Rien ne peut être lu : { $error }
error-open = { $name } ne se lit pas : { $error }
error-seek = Impossible de se déplacer dans ce morceau : { $error }
error-path = { $name } : { $error }
error-and-more = { $error } (et { $count ->
    [one] { $count } autre
   *[other] { $count } autres
    })
