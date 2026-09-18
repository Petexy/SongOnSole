# संगीत — हिन्दी.

app-name = संगीत
app-tagline = आपका संगीत, सोफ़े से।
preview = झलक · एक बनाई हुई लाइब्रेरी

library = आपकी लाइब्रेरी
songs = गाने
albums = एलबम
artists = कलाकार
favourites = पसंदीदा
queue = कतार

songs-subtitle = आपके पास जो कुछ है।
albums-subtitle = एक एलबम चुनिए और आराम से सुनिए।
artists-subtitle = कोई जानी-पहचानी आवाज़ ढूँढिए।
favourites-subtitle = वे गाने जिन पर आप लौटते हैं।
queue-subtitle = आगे क्या बजेगा, क्रम से।
album-subtitle = { $artist }
artist-subtitle = इस कलाकार का सब कुछ।

track = गाना
length = अवधि
current = बज रहा है
song-count = { $count ->
    [one] { $count } गाना
   *[other] { $count } गाने
    }
album-count = { $count ->
    [one] { $count } एलबम
   *[other] { $count } एलबम
    }
position-in-list = { $of } में से { $at }

empty = यहाँ अभी कोई संगीत नहीं है
empty-note = जिस फ़ोल्डर में आपका संगीत है उसे जोड़िए, वहीं से पढ़ लिया जाएगा।
empty-filter = इस शीर्षक के नीचे अभी कुछ नहीं है।
empty-queue = कतार में कुछ नहीं है
empty-queue-note = कोई गाना बजाइए, या विकल्प मेनू से कतार में जोड़िए।
scanning = आपका संगीत पढ़ा जा रहा है…
scanning-note = गाने मिलते ही दिखते जाते हैं। पूरा होने से पहले भी सुन सकते हैं।

now-playing = अभी बज रहा है
up-next = आगे
ready = जब आप कहें
choose-song = अपनी लाइब्रेरी में से कुछ चुनिए।
nothing-queued = इसके बाद कुछ नहीं।
of-album = { $album } से

options = विकल्प
back = वापस
close = बंद करें
open = खोलें
play = बजाएँ
pause = रोकें
add-queue = कतार में जोड़ें
favourite = पसंदीदा में जोड़ें
unfavourite = पसंदीदा से हटाएँ
remove-queue = कतार से हटाएँ
clear-queue = कतार खाली करें
add-folder = संगीत का फ़ोल्डर जोड़ें
add-folder-short = फ़ोल्डर जोड़ें
refresh = लाइब्रेरी फिर से पढ़ें
open-now-playing = अभी चल रहा है

previous = पिछला
next = अगला
shuffle-on = फेरबदल चालू
shuffle-off = फेरबदल बंद
repeat-off = दोहराव बंद
repeat-all = सब दोहराएँ
repeat-one = एक दोहराएँ
mute = म्यूट करें
unmute = अनम्यूट करें

## What order a shelf is listed in

sort-by = क्रम से लगाएँ
order-name = नाम
order-name-backwards = नाम उल्टा
order-artist = कलाकार
order-album = एल्बम
order-newest = नवीनतम पहले
order-oldest = सबसे पुराने पहले

unknown-artist = अज्ञात कलाकार
unknown-album = अज्ञात एलबम
error-save = आपकी सेटिंग सहेजी नहीं जा सकी: { $error }
error-ffprobe = FFmpeg स्थापित नहीं है, इसलिए आपका संगीत पढ़ा नहीं जा सकता: { $error }
error-read = { $name } पढ़ा नहीं जा सका
error-no-audio = { $name } में कोई आवाज़ नहीं है
error-device = कुछ भी बजाया नहीं जा सकता: { $error }
error-open = { $name } बजता नहीं है: { $error }
error-seek = इस गाने में आगे-पीछे नहीं किया जा सकता: { $error }
error-path = { $name }: { $error }
error-and-more = { $error } (और { $count ->
    [one] { $count } अन्य
   *[other] { $count } अन्य
    })
