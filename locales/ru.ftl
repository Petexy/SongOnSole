# Музыка — Русский.

app-name = Музыка
app-tagline = Ваша музыка, с дивана.
preview = ПРЕДПРОСМОТР · выдуманная коллекция

library = ВАША КОЛЛЕКЦИЯ
songs = Композиции
albums = Альбомы
artists = Исполнители
favourites = Избранное
queue = Очередь

songs-subtitle = Всё, что у вас есть.
albums-subtitle = Выберите пластинку и устраивайтесь.
artists-subtitle = Найдите знакомый голос.
favourites-subtitle = Композиции, к которым вы возвращаетесь.
queue-subtitle = Что будет играть, по порядку.
album-subtitle = { $artist }
artist-subtitle = Всё этого исполнителя.

track = КОМПОЗИЦИЯ
length = ДЛИТЕЛЬНОСТЬ
current = Играет
song-count = { $count ->
    [one] { $count } композиция
    [few] { $count } композиции
    [many] { $count } композиций
   *[other] { $count } композиции
    }
album-count = { $count ->
    [one] { $count } альбом
    [few] { $count } альбома
    [many] { $count } альбомов
   *[other] { $count } альбома
    }
position-in-list = { $at } из { $of }

empty = Здесь ещё нет музыки
empty-note = Добавьте папку, где лежит ваша музыка, и она будет прочитана оттуда.
empty-filter = Под этим заголовком пока ничего нет.
empty-queue = В очереди пусто
empty-queue-note = Включите композицию или добавьте её в очередь из меню.
scanning = Читаем вашу музыку…
scanning-note = Композиции появляются по мере того, как их находят. Слушать можно не дожидаясь конца.

now-playing = СЕЙЧАС ИГРАЕТ
up-next = ДАЛЕЕ
ready = Когда скажете
choose-song = Выберите что-нибудь из своей коллекции.
nothing-queued = После этой ничего нет.
of-album = из { $album }

options = Параметры
back = Назад
close = Закрыть
open = Открыть
play = Играть
pause = Пауза
add-queue = Добавить в очередь
favourite = Добавить в избранное
unfavourite = Убрать из избранного
remove-queue = Убрать из очереди
clear-queue = Очистить очередь
add-folder = Добавить папку с музыкой
add-folder-short = Добавить папку
refresh = Прочитать коллекцию заново
open-now-playing = Сейчас играет

previous = Предыдущая
next = Следующая
shuffle-on = Вперемешку
shuffle-off = По порядку
repeat-off = Без повтора
repeat-all = Повторять всё
repeat-one = Повторять одну
mute = Выключить звук
unmute = Включить звук

## What order a shelf is listed in

sort-by = Сортировать по
order-name = Название
order-name-backwards = Название наоборот
order-artist = Исполнитель
order-album = Альбом
order-newest = Сначала новые
order-oldest = Сначала старые

unknown-artist = Неизвестный исполнитель
unknown-album = Неизвестный альбом
error-save = Не удалось сохранить ваши настройки: { $error }
error-ffprobe = FFmpeg не установлен, поэтому вашу музыку не прочитать: { $error }
error-read = Не удалось прочитать { $name }
error-no-audio = В { $name } нет звука
error-device = Ничего не воспроизвести: { $error }
error-open = { $name } не играет: { $error }
error-seek = По этой композиции не перемотать: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (и ещё { $count ->
    [one] { $count }
    [few] { $count }
    [many] { $count }
   *[other] { $count }
    })
