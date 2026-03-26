use std::convert::Infallible;
use std::time::Duration;

use log::{error, info};
use smol::{
    Timer,
    channel::{Receiver, RecvError, Sender},
    future::FutureExt,
    lock::Mutex,
};
use vlc::MediaPlayerAudioEx as _;

use crate::{AppState, BroadcastEvent, PlayerEvent};

// expected input range: 0.0 ~ 1.0
fn adjust_volume(volume: f32) -> i32 {
    if volume < 0.005 {
        0
    } else {
        (((volume - 1.0) * 6.0).exp() * 100.) as i32
    }
}

pub async fn player(
    state: &Mutex<AppState<'_>>,
    player_event_rx: Receiver<PlayerEvent>,
    broadcast_tx: Sender<BroadcastEvent>,
) -> Result<Infallible, RecvError> {
    let vlc_instance = vlc::Instance::new().expect("Failed to create VLC instance");
    let player = vlc::MediaPlayer::new(&vlc_instance).expect("Failed to create VLC MediaPlayer");

    {
        let state = state.lock().await;
        player.set_pause(!state.player.playing);
        let volume = adjust_volume(state.player.volume);
        if player.set_volume(volume).is_err() {
            error!("Failed to set volume: {}", volume);
        }
    }

    let task1 = async {
        loop {
            let event = match player_event_rx.recv().await {
                Ok(event) => event,
                Err(err) => return Err(err),
            };
            match event {
                PlayerEvent::Pause => {
                    player.set_pause(true);
                }
                PlayerEvent::Resume => {
                    player.set_pause(false);
                }
                PlayerEvent::Skip => {
                    player.stop();
                }
                PlayerEvent::SetVolume => {
                    let volume = state.lock().await.player.volume;
                    let volume = adjust_volume(volume.clamp(0.0, 1.0));
                    if player.set_volume(volume).is_err() {
                        error!("Failed to set volume: {}", volume);
                    }
                }
            }
        }
    };

    let task2 = async {
        let mut queue_was_not_empty = true;
        loop {
            let info = {
                loop {
                    {
                        let mut state = state.lock().await;
                        if let Some(info) = state.queue.try_pop().await {
                            state.now_playing = info.clone();
                            break info;
                        }
                    }
                    Timer::after(Duration::from_millis(200)).await;
                }
            };

            if queue_was_not_empty || info.is_some() {
                let _ = broadcast_tx.send(BroadcastEvent::UpdateQueue).await;
            }

            queue_was_not_empty = info.is_some();

            if let Some(info) = info {
                info!("Start playing song (id: {})", info.id);

                let format = info
                    .formats
                    .iter()
                    .filter(|m| m.acodec.clone().is_some_and(|s| s != "none"))
                    .filter(|m| m.vcodec.clone().is_none_or(|s| s == "none"))
                    .reduce(|acc, e| {
                        std::cmp::max_by_key(acc, e, |v| v.quality.unwrap_or(-10.0) as i32)
                    });

                let format = match format {
                    Some(format) => format,
                    None => {
                        error!("No usable format when playing id: {}", info.id);
                        continue;
                    }
                };

                let Some(media) = vlc::Media::new_location(&vlc_instance, &format.url) else {
                    error!("Failed to create new vlc Media");
                    continue;
                };

                player.set_media(&media);
                player.set_time(0);
                if player.play().is_err() {
                    error!("Failed to start playing");
                }
                Timer::after(Duration::from_millis(100)).await;

                while player.is_playing() {
                    Timer::after(Duration::from_millis(100)).await;
                }
                info!("Finished playing song");
            }
            Timer::after(Duration::from_millis(200)).await;
        }
    };

    task1.or(task2).await
}
