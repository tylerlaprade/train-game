# train-game

A side-scrolling terminal steam train for toddlers.

- **Right arrow / Left arrow** — drive the train forward / backward
- **Spacebar** — toot the horn
- Type `quit` or `exit` — quit

<img width="1440" height="900" alt="image" src="https://github.com/user-attachments/assets/ac1c141e-e1c8-4181-a36c-02134198826d" />

<img width="1440" height="900" alt="image" src="https://github.com/user-attachments/assets/93b71633-24cb-4604-b098-1b945824fd98" />

## Download and run

Download the ZIP for your platform from the
[latest release](https://github.com/tylerlaprade/train-game/releases/latest),
extract it, and run the game from a terminal.

### macOS Apple silicon

Use `train-game-macos-arm64.zip`.

```sh
cd ~/Downloads/train-game-macos-arm64
./train-game
```

The macOS release is not signed or notarized yet. If macOS says it cannot verify
the game, remove the quarantine attribute from the extracted release folder:

```sh
xattr -dr com.apple.quarantine ~/Downloads/train-game-macos-arm64
```

### macOS Intel

Use `train-game-macos-x86_64.zip`.

```sh
cd ~/Downloads/train-game-macos-x86_64
./train-game
```

The macOS release is not signed or notarized yet. If macOS says it cannot verify
the game, remove the quarantine attribute from the extracted release folder:

```sh
xattr -dr com.apple.quarantine ~/Downloads/train-game-macos-x86_64
```

### Windows

Use `train-game-windows-x86_64.zip`.

```powershell
cd $HOME\Downloads\train-game-windows-x86_64
.\train-game.exe
```

Run it from Windows Terminal or PowerShell so the terminal stays open.

### Linux

Use `train-game-linux-x86_64.zip`.

```sh
cd ~/Downloads/train-game-linux-x86_64
./train-game
```

If your unzip tool drops executable permissions, restore them:

```sh
chmod +x ~/Downloads/train-game-linux-x86_64/train-game
```

Minimal Linux installs may also need the ALSA runtime libraries for audio.

## Audio

The chugga loop, horn, and "another wheel" voice are vendored under `assets/`
and embedded into the binary at compile time. Rain ambience is generated at
runtime. Playback uses [`rodio`](https://crates.io/crates/rodio), so the game
runs on macOS, Linux, and Windows with no audio toolchain required at runtime.

## Attribution

- **Chugga loop** (`assets/chugga.flac`, from `Steam_engine.ogg`):
  public-domain recording by *aradlaw*, via
  [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Steam_engine.ogg).
- **Whistle / horn** (`assets/whistle.flac`, from `WWS_SteamWhistle.ogg`):
  CC-BY-4.0, via
  [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:WWS_SteamWhistle.ogg).
- **Another wheel** (`assets/another_wheel.flac`):
  generated with the Piper `en_US-ljspeech-medium` voice. The
  [`piper-voices` repository](https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US/ljspeech/medium)
  is MIT licensed, and its model card lists the
  [LJ Speech dataset](https://keithito.com/LJ-Speech-Dataset/) as public domain.

## Requirements

- Rust toolchain (edition 2024 — Rust 1.85+)

## Run

```sh
cargo run --release
```
