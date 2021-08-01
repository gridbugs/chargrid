use crate::error::{Error, Result};
use chargrid_runtime::Size;
use libc;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::io::{AsRawFd, RawFd};

struct WinSize {
    ws_row: libc::c_ushort,
    ws_col: libc::c_ushort,
    _ws_xpixel: libc::c_ushort,
    _ws_ypixel: libc::c_ushort,
}

pub struct LowLevel {
    tty_file: File,
    tty_fd: RawFd,
    original_termios: libc::termios,
}

impl LowLevel {
    fn init_tty(fd: RawFd) -> Result<libc::termios> {
        let mut termios = MaybeUninit::uninit();
        let res = unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        let mut termios = unsafe { termios.assume_init() };
        let original_termios = termios;
        termios.c_iflag &= !(libc::IGNBRK
            | libc::BRKINT
            | libc::PARMRK
            | libc::ISTRIP
            | libc::INLCR
            | libc::IGNCR
            | libc::ICRNL
            | libc::IXON);
        termios.c_oflag &= !libc::OPOST;
        termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
        termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
        termios.c_cflag |= libc::CS8;
        termios.c_cc[libc::VMIN] = 0;
        termios.c_cc[libc::VTIME] = 0;
        let res = unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        Ok(original_termios)
    }

    pub fn new() -> Result<Self> {
        let tty_file = OpenOptions::new().write(true).read(true).open("/dev/tty")?;
        let tty_fd = tty_file.as_raw_fd();
        let original_termios = Self::init_tty(tty_fd)?;
        Ok(Self {
            tty_file,
            original_termios,
            tty_fd,
        })
    }

    pub fn size(&self) -> Result<Size> {
        let mut win_size = WinSize {
            ws_row: 0,
            ws_col: 0,
            _ws_xpixel: 0,
            _ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(self.tty_fd, libc::TIOCGWINSZ.into(), &mut win_size);
        }

        if win_size.ws_row == 0 || win_size.ws_col == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(Size::new(win_size.ws_col as u32, win_size.ws_row as u32))
        }
    }

    pub fn send(&mut self, data: &str) -> io::Result<()> {
        self.tty_file.write_all(data.as_bytes())
    }

    pub fn read_polling(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        self.tty_file.read_to_end(buf)?;
        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        let res = unsafe { libc::tcsetattr(self.tty_fd, libc::TCSAFLUSH, &self.original_termios) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for LowLevel {
    fn drop(&mut self) {
        self.teardown()
            .expect("Failed to reset terminal to original settings");
    }
}
