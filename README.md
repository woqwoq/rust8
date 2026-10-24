# rust8

A CHIP-8 emulator written in Rust, built as a learning project.

## Features

- Full CHIP-8 instruction set (35 opcodes), decoded via pattern matching on the four instruction nibbles.
- Spec-correct memory model: 4 KB RAM, 16 registers, 16-level stack, built-in font loaded at `0x50`, ROMs loaded at `0x200`.
- 64×32 monochrome display rendered through `minifb` at 16× scale, with dirty-flag tracking so frames are only redrawn when pixels change.
- 600 Hz emulated CPU clock stepped inside a 60 FPS render loop (10 instructions per frame).
- Keypad input mapped to a keyboard layout (QWERTY-style), polled each frame via tracked key events.
- Sound via `rodio`: a 440 Hz square wave played while the sound timer is active, gracefully disabled when no audio device is available.
- Graceful, typed error handling throughout (`EmulatorError` wrapping display, processor, and ROM-loading errors with `std::error::Error` source chains).

## Build

```sh
cargo build --release
```

## Usage

```
rust8 --file <ROM>
```

Run one of the bundled example ROMs:

```sh
rust8 --file src/example/IBM_Logo.ch8
rust8 --file src/example/Pong.ch8
rust8 --file src/example/Tetris.ch8
```

Press `Escape` to quit.

### Keypad Mapping

| CHIP-8 Key | Keyboard |
| --- | --- |
| `1 2 3 C` | `1 2 3 4` |
| `4 5 6 D` | `Q W E R` |
| `7 8 9 E` | `A S D F` |
| `A 0 B F` | `Z X C V` |

## Architecture & Project Layout

```
src/
├── main.rs        # Entry point, wires up the CLI runner
├── cli.rs         # clap-based argument parsing & top-level error handling
├── constants.rs   # Hardware constants: RAM, font, screen, clock speed, keymap
├── processor.rs   # CPU: registers, stack, memory, fetch/decode/execute cycle
├── emulator.rs    # Emulator: ties CPU, display, and speaker into the main loop
├── display.rs     # minifb window & framebuffer drawing
└── sound.rs       # rodio square-wave speaker driven by the sound timer
```

### Emulation Loop

Each 60 FPS frame:

1. Poll keypad state from the window.
2. Execute `CPU_CLOCK_HZ / FPS` (10) CPU instructions.
3. Beep if the sound timer is nonzero, otherwise pause the speaker.
4. Tick delay & sound timers at 60 Hz.
5. Redraw the framebuffer only if pixels are dirty.

## ROMs & Attribution

The example ROMs in `src/example/` are not part of this project — they were borrowed from the [loktar00/chip8](https://github.com/loktar00/chip8/tree/master/roms) ROM collection and remain the property of their original authors:

| ROM | Author |
| --- | --- |
| `IBM_Logo.ch8` | IBM (logo demo) |
| `Pong.ch8` | Paul Vervalin, 1990 |
| `Space_Invaders.ch8` | David Winter, 1996 |
| `Tetris.ch8` | Fran Dachille, 1991 |
| `Bowling.ch8` | Humberto R. Salgado, 1979 |
| `Astro_Dodge.ch8` | Revival Studios, 2008 |

All credit for the games goes to their creators; this repository only redistributes them as example ROMs for testing the emulator.
