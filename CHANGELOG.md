# Changelog

How to write a good changelog: [zh-TW](https://keepachangelog.com/zh-TW/1.1.0/), [en](https://keepachangelog.com/en/1.1.0/)

## Unreleased

## v0.2.1

### Changed

- Adjusted volume control to using [Logarithmic Volume Control](https://www.dr-lex.be/info-stuff/volumecontrols.html).

### Fixed

- Be more specific with FFI types to prevent type mismatch on different platform
- Cleanup memory on ffmpeg failed

## v0.2.0

### Changed

- Replace `Symphonia` with `FFMPEG` (using `ffmpeg_next`)
- Print `stderr` of `yt-dlp` to log

### Fixed

- Fix `HttpStream` not actually seeking when calling `std::io::Seek::seek`

## v0.1.1

### Changed

- Frontend will attempt reconnect to service if disconnected
- Small rework on Changelog display

## v0.1.0

### Added

- A partially working frontend capable of controlling backend
- A backend that accepts signals from frontend, and uses `yt-dlp` to fetch & play youtube audio
