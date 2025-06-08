use serde::Deserialize;
use smol::process::{Command, Stdio};

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

pub async fn get_ytdlp(url: String) -> anyhow::Result<Vec<YoutubeInfo>> {
    if matches!(url.chars().next(), None | Some('-')) {
        return Err(anyhow::anyhow!("Invalid URL :{}", url));
    }

    let output = Command::new("yt-dlp")
        .arg("-j")
        .arg("--skip-download")
        .arg("--no-warning")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Call to yt-dlp failed: {}", output.status));
    }

    let result = std::str::from_utf8(&output.stdout)?;

    let list = result
        .lines()
        .map(serde_json::from_str::<YoutubeInfo>)
        .collect::<serde_json::Result<_>>()?;

    Ok(list)
}
