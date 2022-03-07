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

# 只添加引导程序为依赖项，并不足以创建一个可引导的磁盘映像；我们还需要内核编译完成之后，将内核和引导程序组合在一起。然而，截至目前，原生的 cargo 并不支持在编译完成后添加其它步骤（详见这个 issue）。
# 为了解决这个问题，我们建议使用 bootimage 工具——它将会在内核编译完毕后，将它和引导程序组合在一起，最终创建一个能够引导的磁盘映像。我们可以使用下面的命令来安装这款工具：
cargo install bootimage
# 为了运行 bootimage 以及编译引导程序，我们需要安装 rustup 模块 llvm-tools-preview——我们可以使用下面的命令来安装这个工具。
rustup component add llvm-tools-preview
# 成功安装 bootimage 后，创建一个可引导的磁盘映像就变得相当容易。我们来输入下面的命令：
cargo bootimage


# 安装 qemu
brew install qemu
# 启动 hello-os
qemu-system-x86_64 -drive format=raw,file=target/x86_64-hello-os/debug/bootimage-hello-os.bin

# 编译并运行（此命令需要先配置好 .cargo/config )
cargo xrun
```

# cargo bootimage命令背后
在这行命令背后，bootimage 工具执行了三个步骤：
1. 编译我们的内核为一个 ELF（Executable and Linkable Format）文件；
2. 编译引导程序为独立的可执行文件；
3. 将内核 ELF 文件按字节拼接（append by bytes）到引导程序的末端。
当机器启动时，引导程序将会读取并解析拼接在其后的 ELF 文件。这之后，它将把程序片段映射到分页表（page table）中的虚拟地址（virtual address），清零 BSS段（BSS segment），还将创建一个栈。最终它将读取入口点地址（entry point address）——我们程序中 _start 函数的位置——并跳转到这个位置。

# 在真机上运行内核
我们也可以使用 dd 工具把内核写入 U 盘，以便在真机上启动。可以输入下面的命令：

> dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync

在这里，sdX 是U盘的设备名（device name）。请注意，在选择设备名的时候一定要极其小心，因为目标设备上已有的数据将全部被擦除。

写入到 U 盘之后，你可以在真机上通过引导启动你的系统。视情况而定，你可能需要在 BIOS 中打开特殊的启动菜单，或者调整启动顺序。需要注意的是，bootloader 包暂时不支持 UEFI，所以我们并不能在 UEFI 机器上启动。

# I/O 端口
在x86平台上，CPU和外围硬件通信通常有两种方式，内存映射I/O和端口映射I/O。之前，我们已经使用内存映射的方式，通过内存地址0xb8000访问了[VGA文本缓冲区]。该地址并没有映射到RAM，而是映射到了VGA设备的一部分内存上。

与内存映射不同，端口映射I/O使用独立的I/O总线来进行通信。每个外围设备都有一个或数个端口号。CPU采用了特殊的in和out指令来和端口通信，这些指令要求一个端口号和一个字节的数据作为参数（有些这种指令的变体也允许发送u16或是u32长度的数据）。

isa-debug-exit设备使用的就是端口映射I/O。其中， iobase 参数指定了设备对应的端口地址（在x86中，0xf4是一个通常未被使用的端口），而iosize则指定了端口的大小（0x04代表4字节）。


# 如果鼠标被 qemu 捕获了无法控制怎么办？
`cmd + ctrl + opt + G` 释放控制
