# Música — Español.

app-name = Música
app-tagline = Su música, desde el sofá.
preview = VISTA PREVIA · una colección inventada

library = SU COLECCIÓN
songs = Canciones
albums = Álbumes
artists = Artistas
favourites = Favoritos
queue = Cola

songs-subtitle = Todo lo que tiene.
albums-subtitle = Elija un disco. Póngase cómodo.
artists-subtitle = Encuentre una voz conocida.
favourites-subtitle = Las canciones a las que vuelve.
queue-subtitle = Lo que va a sonar, en orden.
album-subtitle = { $artist }
artist-subtitle = Todo de este artista.

track = CANCIÓN
length = DURACIÓN
current = Sonando
song-count = { $count ->
    [one] { $count } canción
   *[other] { $count } canciones
    }
album-count = { $count ->
    [one] { $count } álbum
   *[other] { $count } álbumes
    }
position-in-list = { $at } de { $of }

empty = Todavía no hay música aquí
empty-note = Añada la carpeta donde está su música y se leerá desde allí.
empty-filter = Todavía no hay nada bajo este título.
empty-queue = No hay nada en la cola
empty-queue-note = Reproduzca una canción, o añada una desde el menú de opciones.
scanning = Leyendo su música…
scanning-note = Las canciones aparecen a medida que se encuentran. Puede reproducir antes de que termine.

now-playing = SONANDO AHORA
up-next = A CONTINUACIÓN
ready = Cuando usted quiera
choose-song = Elija algo de su colección.
nothing-queued = Nada después de esta.
of-album = de { $album }

options = Opciones
back = Atrás
close = Cerrar
open = Abrir
play = Reproducir
pause = Pausa
add-queue = Añadir a la cola
favourite = Añadir a favoritos
unfavourite = Quitar de favoritos
remove-queue = Quitar de la cola
clear-queue = Vaciar la cola
add-folder = Añadir una carpeta de música
add-folder-short = Añadir carpeta
refresh = Volver a leer la colección
open-now-playing = Sonando ahora

previous = Anterior
next = Siguiente
shuffle-on = Aleatorio activado
shuffle-off = Aleatorio desactivado
repeat-off = Sin repetición
repeat-all = Repetir todo
repeat-one = Repetir una
mute = Silenciar
unmute = Activar el sonido

## What order a shelf is listed in

sort-by = Ordenar por
order-name = Nombre
order-name-backwards = Nombre al revés
order-artist = Artista
order-album = Álbum
order-newest = Lo más reciente primero
order-oldest = Lo más antiguo primero

unknown-artist = Artista desconocido
unknown-album = Álbum desconocido
error-save = No se han podido guardar sus ajustes: { $error }
error-ffprobe = FFmpeg no está instalado, así que no se puede leer su música: { $error }
error-read = No se ha podido leer { $name }
error-no-audio = No hay sonido en { $name }
error-device = No se puede reproducir nada: { $error }
error-open = { $name } no se reproduce: { $error }
error-seek = No se puede avanzar dentro de esta canción: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (y { $count ->
    [one] { $count } más
   *[other] { $count } más
    })
