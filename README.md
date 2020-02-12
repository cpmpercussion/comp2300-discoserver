# Discoboard emulator

Emulates the discovery board used in COMP2300.

## Using

1. This is a Rust project, so you will need to [install Rust](https://www.rust-lang.org/tools/install). Check the tools work with
```
cargo --version
```

2. Once installed, clone this repo and navigate to the repo root. Run
```
cargo build
```

3. For a discoboard project, add the following to `.vscode/launch.json` configurations. Make sure to use the correct paths. Newer versions of platformio use `.pio/build` instead of `.pioenvs`. Change `debug` to `release` in `serverpath` if you want to use a release build made with `cargo build --release` (use this if you want the best performance, such as when testing audio).
```
{
    "type": "cortex-debug",
    "request": "launch",
    "name": "ARM Emulator Debug",
    "cwd": "${workspaceRoot}",
    "device": "STM32L476vg",
    "executable": "${workspaceRoot}/.pioenvs/disco_l476vg/firmware.elf",
    "servertype": "qemu",
    "preLaunchTask": "PlatformIO: Build",
    "serverpath": "/abs/path/to/project/.../arm-emulator/target/debug/arm-emulator",
    "postLaunchCommands": [
        "-break-insert main"
    ]
}
```
  - Note `serverpath` may be marked as "not allowed". The schema is wrong, it is in fact allowed (tested with `cortex-debug` v0.2.7; this has been fixed in later versions).

4. Select the `ARM Emulator Debug` option in the debug selection and use it like normal.
