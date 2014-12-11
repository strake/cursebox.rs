use core::fmt;
use nul::NulStr;

use Attr;

pub fn write_cursor<W: fmt::Write>(mut w: W, x: usize, y: usize) -> fmt::Result {
    write!(w, "\x1B[{};{}H", y+1, x+1)
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TermWriter<W> {
    pub(crate) funcs: [&'static NulStr; T_FUNCS_NUM],
    last_pos: (usize, usize),
    last_attr: (Attr, Attr),
    pub(crate) w: W,
}

impl<W: fmt::Write> TermWriter<W> {
    #[inline]
    pub const fn new(w: W) -> Self { Self { last_pos: (!0, !0),
                                            last_attr: (Attr { bits: !0 },
                                                        Attr { bits: !0 }),
                                     funcs: [str0_utf8!(""); T_FUNCS_NUM], w } }

    #[inline]
    pub fn write_func(&mut self, func: Func) -> fmt::Result {
        self.w.write_str(&self.funcs[func as usize][..])
    }

    pub fn write_char(&mut self, x: u32, xpos: usize, ypos: usize) -> fmt::Result {
        if (xpos-1, ypos) != self.last_pos { write_cursor(&mut self.w, xpos, ypos)? }
        self.last_pos = (xpos, ypos);
        write!(&mut self.w, "{}",
               match ::core::char::from_u32(x).unwrap_or('\0') { '\0' => ' ', x => x })
    }

    pub fn write_attr(&mut self, fg: Attr, bg: Attr) -> fmt::Result {
        if (fg, bg) == self.last_attr { return Ok(()) }
        let e = str0_utf8!("");
        fn f(a: Attr) -> u16 {
            let a = a & Attr::Default;
            if Attr::Default == a { 9 } else { a.bits }
        }
        write!(&mut self.w, "{}\x1B[3{};4{}m{}{}{}",
               self.funcs[Sgr0 as usize], f(fg), f(bg),
               if fg.contains(Attr::Bold)      { self.funcs[Bold      as usize] } else { e },
               if bg.contains(Attr::Bold)      { self.funcs[Blink     as usize] } else { e },
               if fg.contains(Attr::Underline) { self.funcs[Underline as usize] } else { e },
              )?;
        self.last_attr = (fg, bg);
        Ok(())
    }

    pub fn write_clear(&mut self, cx: usize, cy: usize, fg: Attr, bg: Attr) -> fmt::Result {
        self.write_attr(fg, bg)?;
        self.write_func(ClearScreen)?;
        if !is_cursor_hidden(cx, cy) { write_cursor(&mut self.w, cx, cy)?; }
        //self.w.flush();
        self.invalidate_pos();
        Ok(())
    }

    #[inline]
    pub fn invalidate_pos(&mut self) { self.last_pos = (!0, !0); }
}

use self::Func::*;

#[repr(u8)]
pub enum Func {
    EnterCa,
    ExitCa,
    ShowCursor,
    HideCursor,
    ClearScreen,
    Sgr0,
    Underline,
    Bold,
    Blink,
    Reverse,
    EnterKeypad,
    ExitKeypad,
}

pub const T_FUNCS_NUM: usize = 12;

#[inline]
pub fn is_cursor_hidden(cx: usize, cy: usize) -> bool { (cx, cy) == (!0, !0) }
