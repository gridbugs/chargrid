use std::mem;
use std::ptr;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::os::unix::io::{RawFd, AsRawFd};
use std::time::Duration;
use libc;
use cgmath::Vector2;
use error::{Error, Result};

struct WinSize {
    ws_row: libc::c_ushort,
    ws_col: libc::c_ushort,
    _ws_xpixel: libc::c_ushort,
    _ws_ypixel: libc::c_ushort
}

pub struct UnixBackend {
    tty_file: File,
    tty_fd: RawFd,
    original_termios: libc::termios,
}

impl UnixBackend {
    fn init_tty(fd: RawFd) -> Result<libc::termios> {
        let mut termios = unsafe { mem::uninitialized() };
        let res = unsafe { libc::tcgetattr(fd, &mut termios) };
        if res != 0 {
            return Err(Error::last_os_error());
        }

        let original_termios = termios.clone();

        termios.c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP |
                             libc::INLCR | libc::IGNCR | libc::ICRNL |
                             libc::IXON);
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
        let tty_file = OpenOptions::new()
            .write(true)
            .read(true)
            .open("/dev/tty")?;

        let tty_fd = tty_file.as_raw_fd();
        let original_termios = Self::init_tty(tty_fd)?;

        Ok(Self {
            tty_file,
            original_termios,
            tty_fd,
        })
    }

    pub fn size(&self) -> Result<Vector2<u16>> {
        let mut win_size = WinSize { ws_row: 0, ws_col: 0, _ws_xpixel: 0, _ws_ypixel: 0 };
        unsafe {
            libc::ioctl(self.tty_fd, libc::TIOCGWINSZ, &mut win_size);
        }

        if win_size.ws_row == 0 || win_size.ws_col == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(Vector2::new(win_size.ws_col as u16, win_size.ws_row as u16))
        }
    }

    pub fn send(&mut self, data: &str) -> io::Result<()> {
        self.tty_file.write_all(data.as_bytes())
    }

    pub fn read_polling(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        self.tty_file.read_to_end(buf)?;
        Ok(())
    }

    pub fn read_timeout(&mut self, buf: &mut Vec<u8>, timeout: Duration) -> Result<()> {

        let mut timeout = libc::timespec {
            tv_sec: timeout.as_secs() as libc::time_t,
            tv_nsec: timeout.subsec_nanos() as libc::c_long,
        };

        let mut rfds: libc::fd_set = unsafe { mem::zeroed() };
        unsafe {
            libc::FD_SET(self.tty_fd, &mut rfds);
        }

        let res = unsafe {
            libc::pselect(self.tty_fd + 1, &mut rfds, ptr::null_mut(), ptr::null_mut(), &mut timeout, ptr::null_mut())
        };

        let num_events = if res == -1 {
            return Err(Error::last_os_error());
        } else {
            res
        };

        if num_events > 0 {
            self.read_polling(buf)?;
        }
        Ok(())
    }

    pub fn read_waiting(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        let mut rfds: libc::fd_set = unsafe { mem::zeroed() };
        unsafe {
            libc::FD_SET(self.tty_fd, &mut rfds);
        }

        let res = unsafe {
            libc::pselect(self.tty_fd + 1, &mut rfds, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
        };

        let num_events = if res == -1 {
            return Err(Error::last_os_error());
        } else {
            res
        };

        if num_events > 0 {
            self.read_polling(buf)?;
        }
        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        let res = unsafe {
            libc::tcsetattr(self.tty_fd, libc::TCSAFLUSH, &self.original_termios)
        };

        if res != 0 {
            return Err(Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for UnixBackend {
    fn drop(&mut self) {
        self.teardown().expect("Failed to reset terminal to original settings");
    }
}
