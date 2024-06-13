# fantastic-disco

Rust micro:bit v2 project.

[WebUSB hex flash](https://microbit.org/tools/webusb-hex-flashing/)

## 使い方

リポジトリの人はPR立ててCIに投げるのもおすすめ。または[act](https://github.com/nektos/act)

### インストール

```bash
rustup target add thumbv7em-none-eabihf
sudo apt update
sudo apt install libudev-dev gdb-multiarch minicom binutils-arm-none-eabi
```

### ビルド

```bash
cargo build --release
objcopy -O ihex ./target/thumbv7em-none-eabihf/release/fantastic-disco fantastic-disco.hex
```
