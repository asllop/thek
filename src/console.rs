use core::{
    convert::From,
    ops::{Shl, Index, IndexMut},
    fmt::{
        Write,
        Error
    }
};

/*
EXPERIMENTAL STUFF
 */

use core::sync::atomic::AtomicI32;

pub struct MutableThing {
    pub my_val: AtomicI32
}

pub static MT : MutableThing =  MutableThing { my_val: AtomicI32::new(666) };

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        crate::console::console_print(format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

//TODO: make immutable static and use atomic primitives for interior mutability.

/// Console Driver
pub static mut CONSOLE_WRITER : ConsoleWriter = ConsoleWriter {
    console: Console(ColorScheme {
        foreground: ConsoleColor::White,
        background: ConsoleColor::Red
    }),
    x: 0,
    y: 0
};

/// Define a color
#[derive(Copy, Clone)]
pub enum ConsoleColor {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Purple,
    Brown,
    Gray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightPurple,
    Yellow,
    White
}

impl From<u8> for ConsoleColor {
    fn from(value: u8) -> Self {
        match value & 0xF {
            0 => ConsoleColor::Black,
            1 => ConsoleColor::Blue,
            2 => ConsoleColor::Green,
            3 => ConsoleColor::Cyan,
            4 => ConsoleColor::Red,
            5 => ConsoleColor::Purple,
            6 => ConsoleColor::Brown,
            7 => ConsoleColor::Gray,
            8 => ConsoleColor::DarkGray,
            9 => ConsoleColor::LightBlue,
            10 => ConsoleColor::LightGreen,
            11 => ConsoleColor::LightCyan,
            12 => ConsoleColor::LightRed,
            13 => ConsoleColor::LightPurple,
            14 => ConsoleColor::Yellow,
            15 => ConsoleColor::White,
            _ => ConsoleColor::Black
        }
    }
}

#[derive(Copy, Clone)]
pub struct ColorScheme {
    foreground: ConsoleColor,
    background: ConsoleColor
}

impl ColorScheme {
    pub fn new(foreground: ConsoleColor, background: ConsoleColor) -> Self {
        Self {
            foreground,
            background
        }
    }

    pub fn fg(&self) -> ConsoleColor { self.foreground }

    pub fn bg(&self) -> ConsoleColor { self.background }
}

impl core::default::Default for ColorScheme {
    fn default() -> Self {
        Self {
            foreground: ConsoleColor::Gray,
            background: ConsoleColor::Black
        }
    }
}

/// Define a console char with its properties.
pub struct ConsoleChar {
    character: u8,
    color: ColorScheme
}

impl ConsoleChar {
    pub fn new(character: u8, color: ColorScheme) -> Self {
        Self {
            character,
            color
        }
    }

    pub fn get_char(&self) -> u8 {
        self.character
    }

    pub fn get_color(&self) -> u8 {
        ((self.color.bg() as u8) << 4) | (self.color.fg() as u8)
    }
}

/// Console struct with default colors for foreground and background
pub struct Console(ColorScheme);

impl Console {
    pub fn new(color: ColorScheme) -> Self {
        Self(color)
    }

    pub fn set_char(&self, pos: usize, character: u8) -> Result<(), &'static str> {
        self.set_console_char(pos, ConsoleChar::new(character, self.0))
    }

    pub fn set_console_char(&self, pos: usize, character: ConsoleChar) -> Result<(), &'static str> {
        if pos < self.len() {
            unsafe {
                *((0xB8000 + pos * 2) as *mut u8) = character.get_char();
                *((0xB8000 + pos * 2 + 1) as *mut u8) = character.get_color();
            }
            Ok(())
        }
        else {
            Err("Position out of bounds")
        }
    }

    pub fn len(&self) -> usize { 2000 }
}

impl core::default::Default for Console {
    fn default() -> Self {
        Self(ColorScheme::default())
    }
}

#[derive(Copy, Clone)]
pub enum ConIndex {
    /// X, Y
    Cha(usize, usize),
    /// X, Y
    Col(usize, usize)
}

impl Index<ConIndex> for Console {
    type Output = u8;

    fn index(&self, pos: ConIndex) -> &Self::Output {
        match pos {
            ConIndex::Cha(x, y) => {
                let pos = 80 * y + x;
                if pos >= self.len() { panic!("Index out of bounds"); }
                unsafe {
                    &*((0xB8000 + pos * 2) as *mut u8)
                }
            },
            ConIndex::Col(x, y) => {
                let pos = 80 * y + x;
                if pos >= self.len() { panic!("Index out of bounds"); }
                unsafe {
                    &*((0xB8000 + pos * 2 + 1) as *mut u8)
                }
            }
        }
    }
}

impl IndexMut<ConIndex> for Console {
    fn index_mut(&mut self, pos: ConIndex) -> &mut Self::Output {
        match pos {
            ConIndex::Cha(x, y) => {
                let pos = 80 * y + x;
                if pos >= self.len() { panic!("Index out of bounds"); }
                unsafe {
                    & mut*((0xB8000 + pos * 2) as *mut u8)
                }
            },
            ConIndex::Col(x, y) => {
                let pos = 80 * y + x;
                if pos >= self.len() { panic!("Index out of bounds"); }
                unsafe {
                    & mut*((0xB8000 + pos * 2 + 1) as *mut u8)
                }
            }
        }
    }
}

/// Using shl as a "send message" operator, instead of an actual shift.
/// 
/// Clearly not satisfying the intended usage of this trait, but it's cool being able to do something like:
/// 
/// ```
/// let console = Console::new(ConsoleColor::Green, ConsoleColor::Black);
/// &console << (37, 12, "Hello!");
/// ```
impl Shl<(usize, usize, &str)> for &Console {
    type Output = ();

    fn shl(self, other: (usize, usize, &str)) -> Self::Output {
        let (x, y, msg) = other;
        if x < 80 && y < 25 {
            let pos = 80 * y + x;
            // print chars one by one
            for (i, ch) in msg.as_bytes().iter().enumerate() {
                self.set_char(i + pos, *ch).unwrap_or(());
            }
        }
        ()
    }
}

/*
TODO:
- Create a virtual console, with an arbitrary size and a position in the real console.
- When we write to the console, we update first the internal buffer, and then the real video memory.
- When a scroll happens (by new line), we rewrite the whole buffer.
*/
pub struct ConsoleWriter {
    console: Console,
    x: usize,
    y: usize
}

impl ConsoleWriter {
    pub fn pos(&self) -> usize {
        80 * self.y + self.x
    }

    pub fn inc_pos(&mut self) {
        self.x += 1;
        if self.x >= 80 {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= 25 {
            self.y = 0;
        }
    }

    pub fn line_break(&mut self) {
        self.y += 1;
        self.x = 0;
        if self.pos() >= 80*25 {
            self.x = 0;
            self.y = 0;
        }
    }

    pub fn console(&self) -> &Console {
        &self.console
    }
}

//TODO: create a buffer and scroll all lines up when a new line happens

impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for ch in s.as_bytes() {
            // line break
            if *ch == 0x0Au8 {
                self.line_break();
            }
            else {
                self.console.set_char(self.pos(), *ch).unwrap_or(());
                self.inc_pos();
            }
        }
        Ok(())
    }
}

pub fn console_print(args: core::fmt::Arguments<'_>) {
    //TODO: adquire mutex on CONSOLE_WRITER
    unsafe { core::fmt::write(&mut CONSOLE_WRITER, args).unwrap_or(()); }
}