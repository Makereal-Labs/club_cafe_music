use std::process::{Command, Stdio};
use std::sync::{mpsc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use crate::{AppState, Event};

pub fn player(
    state: &Mutex<AppState>,
    broadcast_tx: mpsc::Sender<Event>,
) {
    let mut queue_was_not_empty = true;
    loop {
        let info = {
            let mut state = state.lock().unwrap();
            let info = state.queue.pop_front();
            state.now_playing = info.clone();
            info
        };

        if queue_was_not_empty || info.is_some() {
            let _ = broadcast_tx.send(Event);
        }

        queue_was_not_empty = info.is_some();

        if let Some(info) = info {
            let format = info
                .formats
                .iter()
                .filter(|m| m.acodec.clone().is_some_and(|s| s != "none"))
                .reduce(|acc, e| {
                    std::cmp::max_by_key(acc, e, |v| v.quality.unwrap_or(-10.0) as i32)
                });

            let format = match format {
                Some(format) => format,
                None => {
                    eprintln!("No usable format when playing id: {}", info.id);
                    continue;
                }
            };

            if let Err(err) = vlc(&format.url) {
                eprintln!("{}", err);
            }
        }
        sleep(Duration::from_millis(200));
    }
}

fn vlc(url: &str) -> anyhow::Result<()> {
    let output = Command::new("cvlc")
        .arg("-A")
        .arg("alsa,none")
        .arg("--no-video")
        .arg(url)
        .arg("vlc://quit")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        let result = std::str::from_utf8(&output.stderr)?;
        return Err(anyhow::anyhow!(
            "Call to vlc failed: {}\n{}",
            output.status,
            result
        ));
    }
    Ok(())
}
