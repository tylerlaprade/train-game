# train-game

A side-scrolling terminal steam train for toddlers.

- **Right arrow / Left arrow** — drive the train forward / backward
- **Spacebar** — toot the horn
- **Type the spoken 3-letter word** — add a wheel. Two wheels = one full car.
  Eight cars = train full → a new train starts below it.
- Type `quit` or `exit` — quit

The smoke billows continuously and drifts realistically based on the train's
motion. The caboose is always the only red car.

## Audio

All three sounds are vendored under `assets/` and embedded into the binary
at compile time. Playback uses [`rodio`](https://crates.io/crates/rodio),
so the game runs on macOS, Linux, and Windows with no audio toolchain
required at runtime.

## Attribution

- **Chugga loop** (`assets/chugga.flac`, from `Steam_engine.ogg`):
  public-domain recording by *aradlaw*, via
  [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Steam_engine.ogg).
- **Whistle / horn** (`assets/whistle.flac`, from `WWS_SteamWhistle.ogg`):
  CC-BY-4.0, via
  [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:WWS_SteamWhistle.ogg).

## Requirements

- Rust toolchain (edition 2024 — Rust 1.85+)

## Run

```sh
cargo run --release
```
