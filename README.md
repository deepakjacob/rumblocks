# rumblocks

A [`dwm`](https://dwm.suckless.org) status bar which changes colors based on the block stats written in pure Rust (async / non-blocking).
Note: `dwm` needs to be patched with [status2d](https://dwm.suckless.org/patches/status2d/).  

![rumblocks-image](rumblocks.png)  

## Features

- [Modular](#modifying-the-blocks)
- Lightweight
- [Suckless](https://suckless.org/philosophy)
- Blocks:
  - Executed asynchronously based on Tokio runtime.
  - No shells scripts, pure Rust
  - Maintainable

## Why `dwmblocks`?

In `dwm`, status bar blocks are executed in an infinite loop,:

```sh
while :; do
    xsetroot -name "$(date)"
    sleep 30
done
```

Using `dwmblocks` allows you to divide the status bar into multiple blocks, each of
which can be updated at its own interval.

## Why `rumblocks`?

Vanilla `dwmblocks` block execution is sequential which may lead to a freeze. 
With `rumblocks`, executes blocks in a aynchronous non-blocking manner.

## Installation

[Install Rust](https://www.rust-lang.org/tools/install) and then clone this repository,

```sh
git clone https://github.com/deepakjacob/rumblocks 
cd rumblocks
cargo build -release
sudo cp target/release/rumblocks /usr/local/bin
```

## Usage

To set `rumblocks` as your status bar, you need to run it as a background
process on startup. One way is to add the following to `~/.xinitrc` or any other startup scripts.

If you use dwm's [autostart](https://dwm.suckless.org/patches/autostart/) add it there.

```sh
rumblocks &
```

### Modifying the blocks

You can define your status bar blocks in `src/main.rs`:

```rust

enum BlockType {
    /* block  -   name interval fn to be executed */
    CpuLoadAverage(String, u32, fn() -> String),
    ...
```
Define what this block is supposed to do:

```rust
    ...
    /* ------------------- Cpu Load Avg block ------------------------*/
    let cpu_load_avg = BlockType::CpuLoadAverage(CPU_LOAD_AVG.to_string(), 5, || {
        let value = if let Ok(loadavg) = System::new().load_average() {
            loadavg.one
        } else {
            0.0
        };
        cpu_load_avg_format(value)
    });
    ...

```
Send it for execution `src/main.rs`

```rust
 block_sender.send(cpu_load_avg).await.unwrap();
 ...
```
Receive the output of the block execution add it to formating :

```rust

let cpu_load_avg = match status_map.get(CPU_LOAD_AVG) {
   Some(cpu_info) => cpu_info,
   _ => "",
};

...
update_status(&format!(
     "{cpu_load_avg} {SEPARATOR} other blocks here...",
      cpu_load_avg, ...other blocks definitions here...
));
```


Each block has the following properties:

| Property        | Description                                                                                             |
| --------------- | ------------------------------------------------------------------------------|
| Block Type Enum | The block you wish to execute in your block.                                  |
| Update interval | Time in seconds, after which you want the block to update, needs to be > 0    |
| Update function | The closure that will be executed for this block                              |


```rust

CpuLoadAverage(String, u32, fn() -> String),

```


Apart from that, you need `dwm` to be patched with
[status2d](https://dwm.suckless.org/patches/status2d/).

## Credits

This work would not have been possible without the following:  
 - [Luke's build of dwmblocks](https://github.com/LukeSmithxyz/dwmblocks)  
 - [statuscmd patch](https://dwm.suckless.org/patches/status2d/)  
 - [Rust Tokio](https://tokio.rs/)  

