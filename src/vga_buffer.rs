use volatile::Volatile;
use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA 缓冲区大小
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Rust 编译器的优化可能让代码不按预期工作。
/// 产生问题的原因在于，我们只向 Buffer 写入，却不再从它读出数据。此时，编译器不知道我们事实上在操作 VGA 缓冲区内存，
/// 而不是在操作普通的 RAM——因此也不知道产生的副效应（side effect），即会有几个字符显示在屏幕上。
/// 这时，编译器也许会认为这些写入操作都没有必要，甚至会选择忽略这些操作！所以，为了避免这些并不正确的优化，这些写入操作应当被指定为易失操作。
/// 这将告诉编译器，这些写入可能会产生副效应，不应该被优化掉。
/// 为了在我们的 VGA 缓冲区中使用易失的写入操作，我们使用 volatile 库。
/// 这个包（crate）提供一个名为 Volatile 的包装类型（wrapping type）和它的 read、write 方法；
/// 这些方法包装了 core::ptr 内的 read_volatile 和 write_volatile 函数，从而保证读操作或写操作不会被编译器优化。
#[repr(transparent)]
struct Buffer {
    // 在这里，我们不使用 ScreenChar ，而选择使用 Volatile<ScreenChar> 
    // 在这里，Volatile 类型是一个泛型（generic），可以包装几乎所有的类型——这确保了我们不会通过普通的写入操作，意外地向它写入数据；转而使用提供的 write 方法。
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// 我们将让这个 Writer 类型将字符写入屏幕的最后一行，并在一行写满或接收到换行符 \n 的时候，将所有的字符向上位移一行。
/// column_position 变量将跟踪光标在最后一行的位置。当前字符的前景和背景色将由 color_code 变量指定；
/// 另外，我们存入一个 VGA 字符缓冲区的可变借用到buffer变量中。需要注意的是，这里我们对借用使用显式生命周期（explicit lifetime），
/// 告诉编译器这个借用在何时有效：我们使用** 'static 生命周期 **（’static lifetime），意味着这个借用应该在整个程序的运行期间有效；
/// 这对一个全局有效的 VGA 字符缓冲区来说，是非常合理的。
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// [Code page 437](https://en.wikipedia.org/wiki/Code_page_437)
    /// VGA 字符缓冲区只支持 ASCII 码字节和代码页 437（Code page 437）定义的字节。Rust 语言的字符串默认编码为 UTF-8，也因此可能包含一些 VGA 字符缓冲区不支持的字节：
    /// 我们使用 match 语句，来区别可打印的 ASCII 码或换行字节，和其它不可打印的字节。对每个不可打印的字节，我们打印一个 ■ 符号；这个符号在 VGA 硬件中被编码为十六进制的 0xfe。
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// 遍历每个屏幕上的字符，把每个字符移动到它上方一行的相应位置。
    /// 我们从第 1 行开始，省略了对第 0 行的枚举过程——因为这一行应该被移出屏幕，即它将被下一行的字符覆写。
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        // 缓冲区写入空格字符来清空一整行的字符。
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// 这个函数首先创建一个指向 0xb8000 地址VGA缓冲区的 Writer。
/// 实现这一点，我们需要编写的代码可能看起来有点奇怪：
/// 首先，我们把整数 0xb8000 强制转换为一个可变的裸指针（raw pointer）；
/// 之后，通过运算符*，我们将这个裸指针解引用；
/// 最后，我们再通过 &mut，再次获得它的可变借用。
/// 这些转换需要 unsafe 语句块（unsafe block），因为编译器并不能保证这个裸指针是有效的。
pub fn print_something() {
    use core::fmt::Write;

    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    // 注意输出：需要注意的是，ö 字符被打印为两个 ■ 字符。"你好" 被打印为六个■
    // 这是因为在 UTF-8 编码下，字符 ö 是由两个字节表述的——而这两个字节并不处在可打印的 ASCII 码字节范围之内。
    // 事实上，这是 UTF-8 编码的基本特点之一：如果一个字符占用多个字节，那么每个组成它的独立字节都不是有效的 ASCII 码字节（the individual bytes of multi-byte values are never valid ASCII）。
    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
    writer.write_string("你好\n");

    // write! 宏返回的 Result 类型必须被使用，所以我们调用它的 unwrap 方法，它将在错误发生时 panic。这里的情况下应该不会发生这样的问题，因为写入 VGA 字符缓冲区并没有可能失败。
    write!(writer, "the numbers are {} and {}!", 1, 1.0/3.0).unwrap();
}