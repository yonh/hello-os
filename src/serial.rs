use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        /// 和 isa-debug-exit设备一样，UART也是用过I/O端口进行编程的。
        /// 由于UART相对来讲更加复杂，它使用多个I/O端口来对不同的设备寄存器进行编程。
        /// 不安全的SerialPort::new函数需要UART的第一个I/O端口的地址作为参数，从该地址中可以计算出所有所需端口的地址。
        /// 我们传递的端口地址为0x3F8 ，该地址是第一个串行接口的标准端口号。
        let mut serial_port = unsafe {
            SerialPort::new(0x3f8)
        };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!( "\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}