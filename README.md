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

3. Follow one of the following to use this to debug a discoboard project. Make sure to use the correct paths instead of the placeholders. Change `debug` to `release` in the emulator path if you want to use a release build made with `cargo build --release` (use this if you want the best performance, such as when testing audio).

    1. If using `cortex-debug` to debug, add the following to `.vscode/launch.json` configurations. TODO: find how cli args are passed, and optionally pass `--audio`.

    ```
    {
        "type": "cortex-debug",
        "request": "launch",
        "name": "ARM Emulator Debug",
        "cwd": "${workspaceRoot}",
        "device": "STM32L476vg",
        "executable": "${workspaceRoot}/.pio/build/disco_l476vg/firmware.elf",
        "servertype": "qemu",
        "preLaunchTask": "PlatformIO: Build",
        "serverpath": "/abs/path/to/comp2300-disco-emulator/target/debug/discoserver",
        "postLaunchCommands": [
            "-break-insert main"
        ]
    }
    ```

    - Note `serverpath` may be marked as "not allowed". The schema is wrong, it is in fact allowed (tested with `cortex-debug` v0.2.7; this has been fixed in later versions).

    2. If using `platformio`, add the following to your `platformio.ini`. You may need to comment out the existing entry to convince platformio to debug using this configuration.

    ```
    [env:emulate]
    platform = ststm32@6.0.0
    board = disco_l476vg
    framework = stm32cube
    build_flags = -g -O0
    debug_tool = custom
    debug_port = localhost:50030; or whatever port you want. Fix the corresponding debug_server arg if changed.
    debug_server =
        /abs/path/to/comp2300-disco-emulator/target/debug/discoserver
        tcp::50030
        ; --audio; uncomment to enable audio
        -kernel
        /abs/path/to/project/.pio/build/emulate/firmware.elf
    debug_init_cmds =
        target remote $DEBUG_PORT
        b main
    ```

4. Select the `ARM Emulator Debug` (or `PIO Debug` if platformio) option in the debug selection and use it like normal.


### Project structure

- `src/main.rs`: The entry point of the `discoserver` executable. Wraps the `disco_emulator` library in a GDB remote protocol compatible server.
- `disco_emulator/src/lib.rs`: The entry point for the emulator itself.
- `tests/*`: Tests for the emulator.


### Tests

Run the tests with `cargo test --all`. Typically each integration test compiles a corresponding program using `arm-none-eabi-as` and `arm-none-eabi-ld`, so make sure these are on your PATH (PlatformIO bundles these in `.platformio/packages/toolchain-gccarmnoneeabi/bin`). It then steps through, checking registers and flags for correct values.
