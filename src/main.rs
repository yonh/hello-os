#![no_std] // 禁止与标准库链接
#![no_main] // 要告诉 Rust 编译器我们不使用预定义的入口点，我们可以添加 #![no_main] 属性。

// 幸运的是，Rust支持通过使用不稳定的自定义测试框架（custom_test_frameworks） 功能来替换默认的测试框架。该功能不需要额外的库，因此在 #[no_std]环境中它也可以工作。
// 它的工作原理是收集所有标注了 #[test_case]属性的函数，然后将这个测试函数的列表作为参数传递给用户指定的runner函数。因此，它实现了对测试过程的最大控制。
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
// reexport_test_harness_main 将测试框架的入口函数的名字设置为 test_main
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;

use core::panic::{PanicInfo, self};

static HELLO: &[u8] = b"Hello World!";




/// 因为编译器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
/// 默认命名为 `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {

    // 测试输出一些内容到屏幕上
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 1, 1.234).unwrap();

    println!("hello world!");

    // 我们将测试框架的入口函数的名字设置为test_main，通过使用条件编译（conditional compilation），我们能够只在上下文环境为测试（test）时调用 test_main，因为该函数将不在非测试上下文中生成。
    #[cfg(test)]
    test_main();


    //panic!("Some panic message");

    loop {}
}


/// 这个函数将在 panic 时被调用 
/// 类型为 PanicInfo 的参数包含了 panic 发生的文件名、代码行数和可选的错误信息。这个函数从不返回，所以他被标记为发散函数（diverging function）。
/// 发散函数的返回类型称作 Never 类型（“never” type），记为!。对这个函数，我们目前能做的很少，所以我们只需编写一个无限循环 loop {}。
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// 我们的runner会打印一个简短的debug信息然后调用列表中的每个测试函数。参数类型 &[&dyn Fn()] 是Fn() trait的 trait object 引用的一个 slice。它基本上可以被看做一个可以像函数一样被调用的类型的引用列表。
/// 由于这个函数在不进行测试的时候没有什么用，这里我们使用 #[cfg(test)]属性保证它只会出现在测试中。
/// 现在当我们运行 cargo xtest ，我们可以发现运行成功了。然而，我们看到的仍然是“Hello World“而不是我们的 test_runner传递来的信息。
/// 这是由于我们的入口点仍然是 _start 函数——自定义测试框架会生成一个main函数来调用test_runner，但是由于我们使用了 #[no_main]并提供了我们自己的入口点，所以这个main函数就被忽略了。
/// 为了修复这个问题，我们需要通过 reexport_test_harness_main 属性来将生成的函数的名称更改为与main不同的名称。然后我们可以在我们的 _start 函数里调用这个重命名的函数:
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
