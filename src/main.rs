#![no_std] // 禁止与标准库链接
#![no_main] // 要告诉 Rust 编译器我们不使用预定义的入口点，我们可以添加 #![no_main] 属性。

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 因为编译器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
    // 默认命名为 `_start`

    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    
    loop {}
}


/// 这个函数将在 panic 时被调用 
/// 类型为 PanicInfo 的参数包含了 panic 发生的文件名、代码行数和可选的错误信息。这个函数从不返回，所以他被标记为发散函数（diverging function）。
/// 发散函数的返回类型称作 Never 类型（“never” type），记为!。对这个函数，我们目前能做的很少，所以我们只需编写一个无限循环 loop {}。
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
