# Músicas — Português (Brasil).

app-name = Músicas
app-tagline = Suas músicas, do sofá.
preview = PRÉVIA · uma coleção inventada

library = SUA COLEÇÃO
songs = Músicas
albums = Álbuns
artists = Artistas
favourites = Favoritas
queue = Fila

songs-subtitle = Tudo o que você tem.
albums-subtitle = Escolha um disco. Fique à vontade.
artists-subtitle = Encontre uma voz conhecida.
favourites-subtitle = As músicas às quais você volta.
queue-subtitle = O que vai tocar, na ordem.
album-subtitle = { $artist }
artist-subtitle = Tudo desse artista.

track = MÚSICA
length = DURAÇÃO
current = Tocando
song-count = { $count ->
    [one] { $count } música
   *[other] { $count } músicas
    }
album-count = { $count ->
    [one] { $count } álbum
   *[other] { $count } álbuns
    }
position-in-list = { $at } de { $of }

empty = Ainda não há músicas aqui
empty-note = Adicione a pasta onde estão suas músicas e elas serão lidas de lá.
empty-filter = Ainda não há nada sob este título.
empty-queue = A fila está vazia
empty-queue-note = Toque uma música, ou acrescente uma pelo menu de opções.
scanning = Lendo suas músicas…
scanning-note = As músicas aparecem conforme são encontradas. Você pode começar a ouvir antes do fim.

now-playing = TOCANDO AGORA
up-next = A SEGUIR
ready = Quando você quiser
choose-song = Escolha algo da sua coleção.
nothing-queued = Nada depois desta.
of-album = de { $album }

options = Opções
back = Voltar
close = Fechar
open = Abrir
play = Tocar
pause = Pausar
add-queue = Acrescentar à fila
favourite = Acrescentar aos favoritos
unfavourite = Tirar dos favoritos
remove-queue = Tirar da fila
clear-queue = Esvaziar a fila
add-folder = Acrescentar uma pasta de músicas
add-folder-short = Acrescentar pasta
refresh = Ler a coleção de novo
open-now-playing = Tocando agora

previous = Anterior
next = Próxima
shuffle-on = Aleatório ligado
shuffle-off = Aleatório desligado
repeat-off = Sem repetição
repeat-all = Repetir tudo
repeat-one = Repetir uma
mute = Silenciar
unmute = Ativar o som

## What order a shelf is listed in

sort-by = Ordenar por
order-name = Nome
order-name-backwards = Nome ao contrário
order-artist = Artista
order-album = Álbum
order-newest = Mais recentes primeiro
order-oldest = Mais antigos primeiro

unknown-artist = Artista desconhecido
unknown-album = Álbum desconhecido
error-save = Não foi possível salvar suas configurações: { $error }
error-ffprobe = O FFmpeg não está instalado, então suas músicas não podem ser lidas: { $error }
error-read = Não foi possível ler { $name }
error-no-audio = Não há som em { $name }
error-device = Nada pode ser tocado: { $error }
error-open = { $name } não toca: { $error }
error-seek = Não dá para avançar dentro desta música: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (e mais { $count ->
    [one] { $count }
   *[other] { $count }
    })
