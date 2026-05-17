# train-game

A side-scrolling terminal steam train for toddlers.

- **Right arrow / Left arrow** — drive the train forward / backward
- **Spacebar** — toot the horn
- **Type the displayed 3-letter word** — add a new train car (up to 8, before the caboose)
- **Esc** or **q** — quit

The smoke billows continuously and drifts realistically based on the train's
motion. The caboose is always the only red car.

## Audio

Sounds are generated on first launch and cached under
`~/Library/Caches/train-game/`. To regenerate, delete that directory and
relaunch. To use your own sounds, drop AIFF files with the same names there.

## Run

```sh
cargo run --release
```
