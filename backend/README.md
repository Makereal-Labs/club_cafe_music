# Makereal Labs cafe music system backend

A simple system for playing music in club, this is the backend code

## Install dependencies

On ubuntu based system:

`sudo apt install -y build-essential cmake libvlc-dev`

To replace pulseaudio with pipewire:

```sh
sudo apt purge --auto-remove pulseaudio
sudo apt install pipewire-pulse pipewire wireplumber
systemctl --user enable pipewire.service pipewire-pulse.service wireplumber.service
```

`yt-dlp` is required to fetch the audio:

```sh
pipx install yt-dlp[default]
# or if you already have yt-dlp installed via pipx
pipx inject yt-dlp yt-dlp[default]
```

(please make an issue if some dependencies are not listed)

## Build & Run

Build: `cargo b`

Build & Run: `cargo r`

Build in release mode: `cargo b -r`

Build & Run in release mode: `cargo r -r`
