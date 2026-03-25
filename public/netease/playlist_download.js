var Log = document.getElementById('text');
Log.innerHTML = '<h1>Playlist Downloader</h1>';
function log(text) {
    Log.innerHTML += text + '<br>';
}

function start_download() {
    log('starting download...');
    const params = new URLSearchParams(window.location.search);
    const id = params.get('id');
    var quality = params.get('quality');
    if (!quality) {
        quality = 'standard';
    }
    var encoding = "mp3";
    if (quality == 'lossless') {
        encoding = "flac";
    }
    if (!id) {
        log('error: no playlist id');
        return;
    } else {
        log('success: playlist id ' + id);
        log('fetching playlist...');
        fetch('../playlist?id=' + id).then(res => res.json()).then(data => {
            log('success: playlist fetched');
            log('downloading playlist...');
            const tracks = data.tracks;
            for (let i = 0; i < tracks.length; i++) {
                const track = tracks[i];
                fetch('../audio/redirect?id=' + track.id + "&quality=" + quality).then(res => res.blob()).then(blob => {
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = track.name.replace(/[<>:"/\\|?*]/g, '_').trim() + '.' + encoding;
                    a.click();
                    URL.revokeObjectURL(url);
                }).catch(err => {
                    log('error: ' + err);
                })
                log('started download track ' + (i + 1) + '/' + tracks.length + ': ' + track.name)
            }
        }).catch(err => {
            log('error: ' + err);
        });
    }
}

window.onload = start_download;