# RFID Duplicates Checker

This small GUI app checks every second if there are any duplicated RFIDs in TXT file. Implemented for a friend over an afternoon.

### Usage

To run locally:

```bash
cargo run --release
```

You can build an executable:

```bash
cargo build --release
```

When building for Windows, [you can remove annoying console window that starts after running `.exe`](https://stackoverflow.com/questions/29763647/how-to-make-a-program-that-does-not-display-the-console-window) by building the project like this (MSVC toolchain):

```bash
cargo rustc --release -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"
```

...or if you are using GCC, then do this instead:


```bash
cargo rustc --release -- -Clink-args="-Wl,--subsystem,windows"
```

### License

MIT