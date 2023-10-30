# rumblocks
A dynamic status bar for [`dwm`](https://dwm.suckless.org), crafted in pure Rust to offer both asynchronous and non-blocking functionality. The bar color adapts based on the status of each block. 

Note:  
    dwm must be patched with the [status2d extension](https://dwm.suckless.org/patches/status2d/).  
    WiFi signal strength is sourced from the [iw utility](https://wireless.wiki.kernel.org/en/users/documentation/iw).  

![Sample rumblocks image](rumblocks.jpg)  

## Features

    Customizable Modules(#modifying-the-blocks)
    Resource-Efficient
    Aligned with [Suckless Philosophy](https://suckless.org/philosophy)
    Blocks:
        Run asynchronously using the Tokio runtime.
        Built entirely in pure Rust, no shell scripts.
        Easy to Maintain

## Why Choose `dwmblocks`?

In the traditional dwm setup, status bar blocks are refreshed in a continuous loop, as shown below:

```sh
while :; do
    xsetroot -name "$(date)"
    sleep 30
done
```

`dwmblocks` enhances this by allowing you to segment the status bar into multiple blocks. Each of these blocks can be updated at its own, individual rate.

## What Sets `rumblocks` Apart?

While the standard dwmblocks executes blocks sequentially, rumblocks takes it to the next level by running blocks asynchronously in a non-blocking fashion.

## Installation

[Install Rust](https://www.rust-lang.org/tools/install) and then clone this repository,

```sh
git clone https://github.com/deepakjacob/rumblocks 
cd rumblocks
cargo build -release
sudo cp target/release/rumblocks /usr/local/bin
```

## How to Use

To set `rumblocks` as your default status bar, you'll need to run it as a background process during system startup. This can be achieved by adding the necessary command to your `~/.xinitrc` or any other startup scripts you might be using.

If you're utilizing dwm's [autostart functionality](https://dwm.suckless.org/patches/autostart/), you can also add the command there.


```sh
rumblocks &
```

### Modifying the blocks

You can define your status bar blocks like the following `src/main.rs`:

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

## Credits

This work would not have been possible without the following:  
 - [Luke's build of dwmblocks](https://github.com/LukeSmithxyz/dwmblocks)  
 - [status2d patch](https://dwm.suckless.org/patches/status2d/)  
 - [Rust Tokio](https://tokio.rs/)  

