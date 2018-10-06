#![no_std]

#![feature(const_fn)]
#![feature(const_str_as_ptr)]

#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]

//! Start with [`UI`](struct.UI.html).
//! See too [examples](https://github.com/strake/cursebox.rs/blob/master/examples).
//!
//! Usual use of the library each frame/iteration is to [`clear`](struct.UI.html#method.clear) the screen, draw, and then [`present`](struct.UI.html#method.present).
//! You can [check events](struct.UI.html#method.fetch_event) before or after.

extern crate buf;
#[macro_use]
extern crate bitflags;
extern crate containers;
#[macro_use]
extern crate derivative;
extern crate io;
extern crate libc;
extern crate loca;
#[macro_use]
extern crate null_terminated as nul;
extern crate ptr;
extern crate slot;
extern crate subslice;
#[macro_use]
extern crate syscall;
extern crate time;
#[macro_use]
extern crate unix;
extern crate unix_signal;
extern crate unix_tty;

use containers::collections::{RawVec, FixedStorage};
use core::{cmp, fmt, mem, num::NonZeroUsize, sync::atomic::{AtomicBool, Ordering as Memord}};
use io::Write;
use libc::c_int;
use loca::Alloc;
use nul::NulStr;
use slot::Slot;
use unix::{file::File, err::OsErr};
use unix_tty::TtyExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub ch: u32,
    pub fg: Attr,
    pub bg: Attr,
}

pub const HideCursor: usize = !0;

bitflags! {
    pub struct Attr: u16 {
        const Black   = 0x00;
        const Red     = 0x01;
        const Green   = 0x02;
        const Yellow  = 0x03;
        const Blue    = 0x04;
        const Magenta = 0x05;
        const Cyan    = 0x06;
        const White   = 0x07;
        const Default = 0x0F;

        const Bold      = 0x10;
        const Underline = 0x20;
    }
}

pub mod input;
pub use input::{Event, Key, Mod};

mod cellbuf;
mod ringbuffer;
mod term;
mod terminfo;
mod utf8;
mod util;

pub use cellbuf::{CellBuf, CellsMut};
use ringbuffer::Ringbuffer;

static mut winch_fds: [c_int; 2] = [-1; 2];

/// Cell-grid TTY UI
///
/// Merely holds a grid of cells which you can modify (with [`cells_mut`](#method.cells_mut) or [`printer`](#method.printer)) and then [`present`](#method.present) to the TTY.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct UI<A: Alloc> {
    cell_buffer: CellBuf<A>,
    term_writer: term::TermWriter<buf::Write<u8, File, FixedStorage<'static, u8>>>,
    cursor_x: usize, cursor_y: usize,
    fg: Attr, bg: Attr,
    term_size: (u16, u16),
    #[derivative(Debug = "ignore")]
    orig_tios: ::libc::termios,
    input_mode: input::Mode,
    inbuf: Ringbuffer<'static>,
    keys: [&'static NulStr; input::TB_KEYS_NUM],
}

static lock: AtomicBool = AtomicBool::new(false);

macro_rules! static_buf {
    [$t:ty; $x:expr] => {{
        static mut buf: Slot<[$t; $x]> = Slot::new();
        &mut buf.x
    }}
}

impl<A: Alloc> UI<A> {
    /// Open "/dev/tty" and make a new `UI` with it.
    pub fn new_in(alloc: A) -> Result<Self, OsErr> {
        use unix::file::*;

        lock.compare_exchange(false, true, Memord::Acquire, Memord::Relaxed)
            .map_err(|_| ::unix::err::EBUSY)?;

        let tty = open_at(None, str0!("/dev/tty"), OpenMode::RdWr, FileMode::empty())?;
        let terminfo::Spec { funcs, keys } = terminfo::init(unsafe { static_buf![u8; 0x4000] })
            .ok_or(OsErr(unsafe { NonZeroUsize::new_unchecked(!0) }))?;
        let (winch_rx, winch_tx) = new_pipe(OpenFlags::empty())?;
        unsafe { winch_fds = [winch_rx.fd() as i32, winch_tx.fd() as i32] };
        mem::forget((winch_rx, winch_tx));

        unsafe extern "C" fn sigwinch_handler(_: usize, _: &::libc::siginfo_t, _: &::libc::ucontext_t) {
            syscall!(WRITE, winch_fds[1], &1u32 as *const u32, 4);
        }
        unsafe { unix_signal::sigaction(::libc::SIGWINCH as _, sigwinch_handler, unix_signal::Flags::empty())?; }

        let orig_tios = tty.get_termios()?;
        let mut ui = Self {
            cell_buffer: CellBuf::new_in(alloc),
            term_writer: unsafe {
                static mut buf: [Slot<u8>; 0x8000] = [Slot::new(); 0x8000];
                term::TermWriter::new(buf::Write::from_raw(tty, RawVec::from_storage(&mut buf[..])))
            },
            cursor_x: !0, cursor_y: !0,
            fg: Attr::Default, bg: Attr::Default,
            term_size: (0, 0),
            orig_tios,
            input_mode: input::Mode::Esc,
            inbuf: Ringbuffer {
                buf: unsafe { static_buf![u8; 0x1000] },
                begin: 0 as _, end: 0 as _,
            },
            keys,
        };
        ui.term_writer.funcs = funcs;
        ui.start()?;
        ui.term_writer.write_clear(ui.cursor_x, ui.cursor_y, ui.fg, ui.bg);
        Ok(ui)
    }

    #[inline] pub fn width(&self) -> usize { self.term_size.0 as _ }
    #[inline] pub fn height(&self) -> usize { self.term_size.1 as _ }

    /// Return a mutable view of the cells, which can be used to draw on the screen.
    #[inline] pub fn cells_mut(&mut self) -> CellsMut { self.cell_buffer.cells_mut().1 }

    #[inline] fn tty_mut(&mut self) -> &mut File { self.term_writer.w.as_mut() }

    /// Restart the UI (after calling `stop`).
    pub fn start(&mut self) -> Result<(), OsErr> {
        let mut tios = self.orig_tios;
        tios.c_iflag &= !(::libc::IGNBRK | ::libc::BRKINT | ::libc::PARMRK | ::libc::ISTRIP |
                          ::libc::INLCR | ::libc::IGNCR | ::libc::ICRNL | ::libc::IXON);
        tios.c_oflag &= !::libc::OPOST;
        tios.c_lflag &= !(::libc::ECHO | ::libc::ECHONL | ::libc::ICANON | ::libc::ISIG | ::libc::IEXTEN);
        tios.c_cflag &= !(::libc::CSIZE | ::libc::PARENB);
        tios.c_cflag |= ::libc::CS8;
        tios.c_cc[::libc::VMIN] = 0;
        tios.c_cc[::libc::VTIME] = 0;
        self.tty_mut().set_termios(tios, ::unix_tty::termios::When::Flush)?;

        use core::fmt::Write;
        use term::Func::*;
        write!(&mut self.term_writer.w, "{}{}{}",
               self.term_writer.funcs[EnterCa     as usize],
               self.term_writer.funcs[EnterKeypad as usize],
               if term::is_cursor_hidden(self.cursor_x, self.cursor_y) {
                   self.term_writer.funcs[HideCursor as usize]
               } else { str0_utf8!("") });
        self.term_writer.w.flush();

        self.update_size()?;
        self.inbuf.clear();
        Ok(())
    }

    /// Stop the UI temporarily and revert the term to its initial state.
    pub fn stop(&mut self) {
        use core::fmt::Write;
        use term::Func::*;
        write!(&mut self.term_writer.w, "{}{}{}{}{}",
               self.term_writer.funcs[ShowCursor  as usize],
               self.term_writer.funcs[Sgr0        as usize],
               self.term_writer.funcs[ClearScreen as usize],
               self.term_writer.funcs[ExitCa      as usize],
               self.term_writer.funcs[ExitKeypad  as usize],
              );
        self.term_writer.w.flush();
        let tios = self.orig_tios;
        self.tty_mut().set_termios(tios, ::unix_tty::termios::When::Flush);
    }

    fn update_size(&mut self) -> Result<(), OsErr> {
        self.term_size = self.tty_mut().get_tty_size()?;
        self.cell_buffer.resize(self.term_size.0 as _, self.term_size.1 as _).map_err(|_| ::unix::err::ENOMEM)?;
        self.cell_buffer.cells_mut().0.clear(self.fg, self.bg);
        self.term_writer.write_clear(self.cursor_x, self.cursor_y, self.fg, self.bg);
        self.term_writer.w.flush();
        Ok(())
    }

    /// Clear the UI. You likely want to call this before you begin drawing.
    #[inline]
    pub fn clear(&mut self) { self.cell_buffer.cells_mut().1.clear(self.fg, self.bg) }

    /// Show the present state of the UI on the TTY.
    pub fn present(&mut self) {
        self.term_writer.invalidate_pos();

        for y in 0..self.cell_buffer.height() {
            for x in 0..self.cell_buffer.width() {
                let (front, back) = unsafe {
                    let (mut front, back) = self.cell_buffer.cells_mut();
                    (front.at_unchecked_mut(x, y), back.at_unchecked(x, y))
                };
                if *back == *front { continue }
                self.term_writer.write_attr(back.fg, back.bg);
                self.term_writer.write_char(back.ch, x, y);
                *front = *back;
            }
        }
        if !term::is_cursor_hidden(self.cursor_x as _, self.cursor_y as _) {
            term::write_cursor(&mut self.term_writer.w, self.cursor_x as _, self.cursor_y as _);
        }
        self.term_writer.w.flush();
    }

    /// Fetch the next event from the TTY.
    #[inline]
    pub fn fetch_event(&mut self, timeout: Option<::time::Span>) -> Result<Option<input::Event>, OsErr> { unsafe {
        let tty_fd = self.tty_mut().fd() as ::libc::c_int;

        use input::extract_event;

        if let Some((mod_, key)) = extract_event(&mut self.inbuf, self.input_mode, mem::transmute(self.keys)) { return Ok(Some(Event::Key(mod_, key))) }

        match self.inbuf.push_from_file(self.term_writer.w.as_mut()) {
            Err(::unix::err::EAGAIN) | Err(::unix::err::EWOULDBLOCK) => return Ok(None),
            Err(e) => return Err(e),
            Ok(n) if n > 0 => if let Some((mod_, key)) = extract_event(&mut self.inbuf, self.input_mode, mem::transmute(self.keys)) { return Ok(Some(Event::Key(mod_, key))) },
            _ => ()
        }

        // 0 == r || not enough data
        loop {
            let mut events = mem::zeroed();
            ::libc::FD_SET(tty_fd, &mut events);
            ::libc::FD_SET(winch_fds[0], &mut events);
            esyscall!(SELECT, cmp::max(tty_fd, winch_fds[0])+1, &mut events as *mut _, 0, 0,
                      timeout.and_then(::time::Span::to_c_timespec)
                             .map(|::libc::timespec { tv_sec, tv_nsec }| ::libc::timeval { tv_sec, tv_usec: tv_nsec / 1000 })
                             .as_mut().map_or(0 as *const ::libc::timeval, |p| p as *const _))?;
            if ::libc::FD_ISSET(tty_fd, &events as *const _ as *mut _) {
                if 0 == self.inbuf.push_from_file(self.term_writer.w.as_mut())? { continue }
                if let Some((mod_, key)) = extract_event(&mut self.inbuf, self.input_mode, mem::transmute(self.keys)) { return Ok(Some(Event::Key(mod_, key))) }
            }
            if ::libc::FD_ISSET(winch_fds[0], &events as *const _ as *mut _) {
                esyscall!(READ, winch_fds[0], &mut mem::uninitialized::<[u8; 4]>() as *mut _, 4)?;
                self.update_size();
                return Ok(Some(Event::Resize(self.term_size.0 as _, self.term_size.1 as _)))
            }
        }
    } }

    pub fn set_cursor(&mut self, cx: usize, cy: usize) {
        use term::is_cursor_hidden;
        if is_cursor_hidden(self.cursor_x, self.cursor_y) && !is_cursor_hidden(cx, cy) {
            self.term_writer.write_func(term::Func::ShowCursor);
        }
        if !is_cursor_hidden(self.cursor_x, self.cursor_y) && is_cursor_hidden(cx, cy) {
            self.term_writer.write_func(term::Func::HideCursor);
        }
        self.cursor_x = cx;
        self.cursor_y = cy;
        if !is_cursor_hidden(cx, cy) { term::write_cursor(&mut self.term_writer.w, cx, cy); }
    }

    #[inline]
    pub fn get_cursor(&self) -> (usize, usize) { (self.cursor_x, self.cursor_y) }

    #[inline]
    pub fn input_mode_mut(&mut self) -> &mut input::Mode { &mut self.input_mode }

    /// Set the input mode and return the former input mode.
    #[deprecated(note = "use `input_mode_mut`")]
    #[inline]
    pub fn select_input_mode(&mut self, mode: Option<input::Mode>) -> input::Mode {
        if let Some(mode) = mode { mem::replace(&mut self.input_mode, mode) } else { self.input_mode }
    }

    /// Return a `Printer` which writes to the screen with the given attributes beginning at
    /// `pt`.
    #[inline]
    pub fn printer(&mut self, pt: (usize, usize), fg: Attr, bg: Attr) -> Printer {
        Printer { pt, fg, bg, screen: self.cells_mut() }
    }
}

impl<A: Alloc> Drop for UI<A> {
    #[inline]
    fn drop(&mut self) { unsafe {
        self.stop();
        File::new_unchecked(winch_fds[0] as _);
        File::new_unchecked(winch_fds[1] as _);
        lock.store(false, Memord::Release);
    } }
}

/// Writer to screen
///
/// Cuts off rather than wraps overflow
#[derive(Debug)]
pub struct Printer<'a> {
    pt: (usize, usize),
    fg: Attr, bg: Attr,
    screen: CellsMut<'a>,
}

impl<'a> Printer<'a> {
    #[inline]
    fn print_chars<Xs: Iterator<Item = char>>(&mut self, xs: Xs) -> usize {
        let mut n = 0;
        for x in xs {
            if let Some(p) = self.screen.at_mut(self.pt.0+n, self.pt.1) {
                *p = Cell { ch: x as _, fg: self.fg, bg: self.bg }
            }
            n += 1;
        }
        n
    }
}

impl<'a> fmt::Write for Printer<'a> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.pt.0 += self.print_chars(s.chars());
        Ok(())
    }
}
