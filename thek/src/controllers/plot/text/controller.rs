use core::{
    fmt::{
        Write,
        Error
    },
    default::Default
};

use crate::{
    controllers::plot::text::ansi::AnsiColor,
    devices::plot::text::PlotTextDevice
};

use crate::sys::{
    KLock, KError
};

/// Plot text controller.
pub struct PlotTextController<'a, T: PlotTextDevice<'a>> {
    cols: usize,
    rows: usize,
    x: usize,
    y: usize,
    device_lock: KLock<'a, T>,
    text_color: AnsiColor,
    bg_color: AnsiColor
}

impl<'a, T: PlotTextDevice<'a>> PlotTextController<'a, T> {
    pub fn new(text_color: AnsiColor, bg_color: AnsiColor) -> Self {
        let device_lock = T::mutex().acquire();
        device_lock.enable_cursor().unwrap_or(());
        let (cols, rows) = device_lock.get_size().unwrap_or((0,0));
        let (x, y) = device_lock.get_cursor().unwrap_or((0,0));
        Self {
            cols, rows,
            x, y,
            device_lock,
            text_color, bg_color
        }
    }

    pub fn get_xy(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn set_xy(&mut self, x: usize, y: usize) -> Result<(), KError> {
        self.x = x;
        self.y = y;
        let (_, text_color, bg_color) = self.device_lock.read(x, y)?;
        if let (AnsiColor::Black, AnsiColor::Black) = (text_color, bg_color) {
            self.device_lock.set_color(x, y, self.text_color, self.bg_color)?;
        }
        self.device_lock.set_cursor(x, y)?;
        Ok(())
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.cols, self.rows)
    } 

    fn inc_pos(&mut self) {
        self.x += 1;
        if self.x >= self.cols {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.rows {
            self.y = 0;
        }
    }

    fn scroll_up(&self) {
        // Copy all lines one line up (from 1 to rows-1)
        for line_num in 1..self.rows {
            for char_num in 0..self.cols {
                if let Ok((ch, text_color, bg_color)) = self.device_lock.read(char_num, line_num) {
                    self.device_lock.print(char_num, line_num - 1, text_color, bg_color, ch).unwrap_or_default();
                }
            }
        }
        // Set last line empty
        for char_num in 0..self.cols {
            self.device_lock.print(char_num, self.rows - 1, self.text_color, self.bg_color, 0u8).unwrap_or_default();
        }
    }

    fn line_break(&mut self) {
        if self.y + 1 >= self.rows {
            self.scroll_up();
            self.x = 0;
        }
        else {
            self.y += 1;
            self.x = 0;
        }
    }
}

impl<'a, T: PlotTextDevice<'a>> Default for PlotTextController<'a, T> {
    fn default() -> Self {
        Self::new(AnsiColor::White, AnsiColor::Black)
    }
}

//TODO: parse ANSI commands in the string to set colors, move cursor, etc

impl<'a, T: PlotTextDevice<'a>> Write for PlotTextController<'a, T> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for ch in s.as_bytes() {
            if *ch == '\n' as u8 {
                self.line_break();
            }
            else if *ch == '\t' as u8 {
                let tab_num = self.x / 4;
                self.x = (tab_num  + 1) * 4;
                if self.x >= self.cols {
                    self.x = self.cols - 1;
                }
            }
            else {
                if let Err(_) = self.device_lock.print(self.x, self.y, self.text_color, self.bg_color, *ch) {
                    return Err(Error);
                }
                self.inc_pos();
            }
        }
        if let Err(_) = self.set_xy(self.x, self.y) {
            return Err(Error);
        }
        Ok(())
    }
}
