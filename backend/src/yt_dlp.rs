use serde::Deserialize;
use smol::process::{Command, Stdio};

pub enum YtdlpResult {
    Single(YoutubeInfo),
    Playlist(Vec<YoutubePlaylistEntry>),
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct YoutubeInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub channel: String,
    pub channel_url: String,
    pub duration: u32,
    pub playlist: Option<String>,
    pub thumbnail: String,
    pub formats: Vec<MediaFormat>,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct MediaFormat {
    pub format_note: Option<String>,
    pub quality: Option<f32>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub video_ext: String,
    pub audio_ext: String,
    pub ext: String,
    pub url: String,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct YoutubePlaylistEntry {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub channel: String,
    pub channel_url: String,
    pub duration: u32,
    pub playlist: Option<String>,
}

pub async fn get_ytdlp(url: String) -> anyhow::Result<YtdlpResult> {
    if matches!(url.chars().next(), None | Some('-')) {
        return Err(anyhow::anyhow!("Invalid URL :{}", url));
    }

    let output = Command::new("yt-dlp")
        .arg("-j")
        .arg("--flat-playlist")
        .arg("--skip-download")
        .arg("--no-warning")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Call to yt-dlp failed: {}\n{}",
            output.status,
            std::str::from_utf8(&output.stderr)?
        ));
    }

    let result = std::str::from_utf8(&output.stdout)?;

    let list: Vec<serde_json::Value> = result
        .lines()
        .map(serde_json::from_str)
        .collect::<serde_json::Result<_>>()?;

    let is_playlist = if let Some(first) = list.first() {
        let serde_json::Value::Object(map) = first else {
            return Err(anyhow::anyhow!("yt-dlp did not return a JSON Object"));
        };

        map.get("playlist").is_some_and(|x| !x.is_null())
    } else {
        // Empty playlist
        return Ok(YtdlpResult::Playlist(Vec::new()));
    };

    if is_playlist {
        let playlist = list
            .into_iter()
            .map(serde_json::from_value)
            .collect::<serde_json::Result<_>>()?;
        Ok(YtdlpResult::Playlist(playlist))
    } else {
        let info = serde_json::from_value(list.into_iter().next().unwrap())?;
        Ok(YtdlpResult::Single(info))
    }
}
