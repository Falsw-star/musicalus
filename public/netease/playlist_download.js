var Log = document.getElementById('text');
Log.innerHTML = '<h1>Playlist Downloader</h1>';
function log(text) {
    Log.innerHTML += text + '<br>';
}

async function downloadWithConcurrencyLimit(tracks, quality, encoding, concurrencyLimit = 3) {
    const results = [];
    
    for (let i = 0; i < tracks.length; i += concurrencyLimit) {
        const batch = tracks.slice(i, i + concurrencyLimit);
        
        const promises = batch.map(async (track) => {
            try {
                log('started download track ' + (i + batch.indexOf(track) + 1) + '/' + tracks.length + ': ' + track.name);
                
                const response = await fetch('../audio/redirect?id=' + track.id + "&quality=" + quality);
                const blob = await response.blob();
                if (blob.size < 1000) {
                    throw new Error('Maybe the track is not available.');
                }
                
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = track.name.replace(/[<>:"/\\|?*]/g, '_').trim() + '.' + encoding;
                a.click();
                URL.revokeObjectURL(url);
                
                return { success: true, track: track.name };
            } catch (err) {
                log('error downloading track ' + track.name + ': ' + err);
                return { success: false, track: track.name, error: err };
            }
        });
        
        const batchResults = await Promise.all(promises);
        results.push(...batchResults);
        
        if (i + concurrencyLimit < tracks.length) {
            await new Promise(resolve => setTimeout(resolve, 500));
        }
    }
    
    return results;
}

async function start_download() {
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
        try {
            const response = await fetch('../playlist?id=' + id);
            const data = await response.json();
            log('success: playlist fetched with ' + data.tracks.length + ' tracks');
            
            log('downloading playlist...');
            const tracks = data.tracks;
            
            const results = await downloadWithConcurrencyLimit(tracks, quality, encoding, 3);
            
            const successful = results.filter(r => r.success).length;
            const failed = results.length - successful;
            log(`download complete: ${successful} successful, ${failed} failed`);
            
        } catch (err) {
            log('error: ' + err);
        }
    }
}

window.onload = start_download;