```shell
# 查看支持的编译目标系统
rustc --print target-list
# 添加编译目标系统支持
rustup target add thumbv7em-none-eabihf
# 指定交叉编译目标
cargo build --target thumbv7em-none-eabihf
# 在当前目录使用 nightly 版本的 Rust
rustup override add nightly
# cargo xbuild 封装了 cargo build
# 但不同的是，它将自动交叉编译 core 库和一些编译器内建库（compiler built-in libraries）
cargo install cargo-xbuild
# 下载rust源码
rustup component add rust-src
# 编译
cargo xbuild
```

