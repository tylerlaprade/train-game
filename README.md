# train-game

A side-scrolling terminal steam train for toddlers.

- **Right arrow / Left arrow** — drive the train forward / backward
- **Spacebar** — toot the horn
- **Type the spoken 3-letter word** — add a wheel. Two wheels = one full car.
  Eight cars = train full → a new train starts below it.
- **Esc** or **q** — quit

The smoke billows continuously and drifts realistically based on the train's
motion. The caboose is always the only red car.

## Audio

On first launch the game downloads two short OGG files from Wikimedia
Commons (~1 MB total), transcodes them to AIFF with `ffmpeg`, and caches
them under `~/Library/Caches/train-game/`. 

If `curl` or `ffmpeg` aren't available, the SFX silently fall back to TTS.

To regenerate, delete the cache directory and relaunch. To use your own
sounds, drop AIFF files with the same names there.

## Attribution

- **Chugga loop** (`steam_engine.ogg`): public-domain recording by *aradlaw*,
  via [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Steam_engine.ogg).
- **Whistle / horn** (`WWS_SteamWhistle.ogg`): CC-BY-4.0, via
  [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:WWS_SteamWhistle.ogg).

## Requirements

- Rust toolchain (edition 2024 — Rust 1.85+)
- macOS (uses `afplay` for playback)
- `curl` and `ffmpeg` for first-run audio setup (optional — falls back to TTS)

## Run

```sh
cargo run --release
```
