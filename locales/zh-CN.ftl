# 音乐 — 简体中文。

app-name = 音乐
app-tagline = 您的音乐，隔着房间也能听。
preview = 预览 · 虚构的音乐库

library = 您的音乐库
songs = 歌曲
albums = 专辑
artists = 艺人
favourites = 收藏
queue = 队列

songs-subtitle = 您所有的歌曲。
albums-subtitle = 挑一张专辑，坐下来听。
artists-subtitle = 找一个熟悉的声音。
favourites-subtitle = 您一再回头听的歌。
queue-subtitle = 接下来要播放的，按顺序。
album-subtitle = { $artist }
artist-subtitle = 这位艺人的全部。

track = 歌曲
length = 时长
current = 正在播放
song-count = { $count ->
   *[other] { $count } 首歌
    }
album-count = { $count ->
   *[other] { $count } 张专辑
    }
position-in-list = 第 { $at } / { $of }

empty = 这里还没有音乐
empty-note = 添加存放音乐的文件夹，就会从那里读取。
empty-filter = 这个标题下还没有内容。
empty-queue = 队列是空的
empty-queue-note = 播放一首歌，或者从选项菜单里加入队列。
scanning = 正在读取您的音乐…
scanning-note = 找到一首就显示一首。不必等它读完就可以开始听。

now-playing = 正在播放
up-next = 接下来
ready = 随时可以开始
choose-song = 从您的音乐库里选一首。
nothing-queued = 这首之后没有了。
of-album = 出自 { $album }

options = 选项
back = 返回
close = 关闭
open = 打开
play = 播放
pause = 暂停
add-queue = 加入队列
favourite = 加入收藏
unfavourite = 取消收藏
remove-queue = 移出队列
clear-queue = 清空队列
add-folder = 添加音乐文件夹
add-folder-short = 添加文件夹
refresh = 重新读取音乐库
open-now-playing = 正在播放的歌曲

previous = 上一首
next = 下一首
shuffle-on = 随机开
shuffle-off = 随机关
repeat-off = 不重复
repeat-all = 全部重复
repeat-one = 单曲重复
mute = 静音
unmute = 取消静音

## What order a shelf is listed in

sort-by = 排序方式
order-name = 名称
order-name-backwards = 名称倒序
order-artist = 艺人
order-album = 专辑
order-newest = 最新的在前
order-oldest = 最旧的在前

unknown-artist = 未知艺人
unknown-album = 未知专辑
error-save = 无法保存您的设置：{ $error }
error-ffprobe = 没有安装 FFmpeg，因此读不了您的音乐：{ $error }
error-read = 读不了 { $name }
error-no-audio = { $name } 里没有声音
error-device = 什么也播放不了：{ $error }
error-open = { $name } 放不出来：{ $error }
error-seek = 这首歌无法拖动：{ $error }
error-path = { $name }：{ $error }
error-and-more = { $error }（还有 { $count ->
   *[other] { $count }
    } 个）
