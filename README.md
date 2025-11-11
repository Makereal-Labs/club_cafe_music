# Makereal Labs cafe music system

A simple system for playing music in club

The frontend and backend code were previously in separate repository. The code is now merged in this repository, and the old ones are archived.

## Build

Refer to [Frontend `README.md`](/frontend/README.md) and [Backend `README.md`](/backend/README.md) for more information

### Extra information when building on Raspberry Pi

Dependency `ffmpeg_next` requires enabling feature `rpi` in order to build successfully, you can enable this with `cargo add ffmpeg_next -F rpi`.

If you encounter the error message `ClangDiagnostic("/usr/include/limits.h:124:16: fatal error: 'limits.h' file not found\n")`, try `apt install clang`.

The `ffmpeg` package on Raspberry Pi's apt repository is old. If you need a newer build, consider [build from source](https://jollejolles.github.io/pirecorder/other/install-ffmpeg-raspberry-pi.html). Note that adding `--disable-asm` when configuring `ffmpeg` helps avoid assembler error.

## License

This repository is licensed with the [MIT License](/LICENSE)
