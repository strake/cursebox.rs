#![allow(safe_extern_statics)]

use core::slice;
use io::Read;
use nul::{Nul, NulStr};
use subslice::SubsliceExt;
use unix::env::environ;
use util::SliceExt;

use term::T_FUNCS_NUM;

const TB_KEYS_NUM: usize = 22;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec<'a> {
    pub keys: [&'a NulStr; TB_KEYS_NUM],
    pub funcs: [&'a NulStr; T_FUNCS_NUM],
}

mod spec {
    use super::Spec;

    macro_rules! s {
        [$($x:expr),*] => ([$(str0_utf8!($x)),*]);
        [$($x:expr,)*] => ([$(str0_utf8!($x)),*]);
    }

    pub const rxvt_256color: Spec = Spec {
        keys: s!["\x1B[11~","\x1B[12~","\x1B[13~","\x1B[14~","\x1B[15~","\x1B[17~","\x1B[18~","\x1B[19~","\x1B[20~","\x1B[21~","\x1B[23~","\x1B[24~","\x1B[2~","\x1B[3~","\x1B[7~","\x1B[8~","\x1B[5~","\x1B[6~","\x1B[A","\x1B[B","\x1B[D","\x1B[C"],
        funcs: s!["\x1B7\x1B[?47h", "\x1B[2J\x1B[?47l\x1B8", "\x1B[?25h", "\x1B[?25l", "\x1B[H\x1B[2J", "\x1B[m", "\x1B[4m", "\x1B[1m", "\x1B[5m", "\x1B[7m", "\x1B=", "\x1B>",],
    };

    pub const eterm: Spec = Spec {
        keys: s!["\x1B[11~","\x1B[12~","\x1B[13~","\x1B[14~","\x1B[15~","\x1B[17~","\x1B[18~","\x1B[19~","\x1B[20~","\x1B[21~","\x1B[23~","\x1B[24~","\x1B[2~","\x1B[3~","\x1B[7~","\x1B[8~","\x1B[5~","\x1B[6~","\x1B[A","\x1B[B","\x1B[D","\x1B[C"],
        funcs: s!["\x1B7\x1B[?47h", "\x1B[2J\x1B[?47l\x1B8", "\x1B[?25h", "\x1B[?25l", "\x1B[H\x1B[2J", "\x1B[m", "\x1B[4m", "\x1B[1m", "\x1B[5m", "\x1B[7m", "", "",],
    };

    pub const screen: Spec = Spec {
        keys: s!["\x1BOP","\x1BOQ","\x1BOR","\x1BOS","\x1B[15~","\x1B[17~","\x1B[18~","\x1B[19~","\x1B[20~","\x1B[21~","\x1B[23~","\x1B[24~","\x1B[2~","\x1B[3~","\x1B[1~","\x1B[4~","\x1B[5~","\x1B[6~","\x1BOA","\x1BOB","\x1BOD","\x1BOC"],
        funcs: s!["\x1B[?1049h", "\x1B[?1049l", "\x1B[34h\x1B[?25h", "\x1B[?25l", "\x1B[H\x1B[J", "\x1B[m", "\x1B[4m", "\x1B[1m", "\x1B[5m", "\x1B[7m", "\x1B[?1h\x1B=", "\x1B[?1l\x1B>",],
    };

    pub const rxvt_unicode: Spec = Spec {
        keys: s!["\x1B[11~","\x1B[12~","\x1B[13~","\x1B[14~","\x1B[15~","\x1B[17~","\x1B[18~","\x1B[19~","\x1B[20~","\x1B[21~","\x1B[23~","\x1B[24~","\x1B[2~","\x1B[3~","\x1B[7~","\x1B[8~","\x1B[5~","\x1B[6~","\x1B[A","\x1B[B","\x1B[D","\x1B[C"],
        funcs: s!["\x1B[?1049h", "\x1B[r\x1B[?1049l", "\x1B[?25h", "\x1B[?25l", "\x1B[H\x1B[2J", "\x1B[m\x1B(B", "\x1B[4m", "\x1B[1m", "\x1B[5m", "\x1B[7m", "\x1B=", "\x1B>",],
    };

    pub const linux: Spec = Spec {
        keys: s!["\x1B[[A","\x1B[[B","\x1B[[C","\x1B[[D","\x1B[[E","\x1B[17~","\x1B[18~","\x1B[19~","\x1B[20~","\x1B[21~","\x1B[23~","\x1B[24~","\x1B[2~","\x1B[3~","\x1B[1~","\x1B[4~","\x1B[5~","\x1B[6~","\x1B[A","\x1B[B","\x1B[D","\x1B[C"],
        funcs: s!["", "", "\x1B[?25h\x1B[?0c", "\x1B[?25l\x1B[?1c", "\x1B[H\x1B[J", "\x1B[0;10m", "\x1B[4m", "\x1B[1m", "\x1B[5m", "\x1B[7m", "", "",],
    };

    pub const xterm: Spec = Spec {
        keys: s!["\x1BOP","\x1BOQ","\x1BOR","\x1BOS","\x1B[15~","\x1B[17~","\x1B[18~","\x1B[19~","\x1B[20~","\x1B[21~","\x1B[23~","\x1B[24~","\x1B[2~","\x1B[3~","\x1BOH","\x1BOF","\x1B[5~","\x1B[6~","\x1BOA","\x1BOB","\x1BOD","\x1BOC"],
        funcs: s!["\x1B[?1049h", "\x1B[?1049l", "\x1B[?12l\x1B[?25h", "\x1B[?25l", "\x1B[H\x1B[2J", "\x1B(B\x1B[m", "\x1B[4m", "\x1B[1m", "\x1B[5m", "\x1B[7m", "\x1B[?1h\x1B=", "\x1B[?1l\x1B>",],
    };
}

static terms: &'static [(&'static str, Spec)] = &[
    ("rxvt-256color", spec::rxvt_256color),
    ("Eterm", spec::eterm),
    ("screen", spec::screen),
    ("rxvt-unicode", spec::rxvt_unicode),
    ("linux", spec::linux),
    ("xterm", spec::xterm),
];

static terms_compat: &'static [(&'static str, Spec)] = &[
    ("xterm", spec::xterm),
    ("rxvt", spec::rxvt_unicode),
    ("linux", spec::linux),
    ("Eterm", spec::eterm),
    ("screen", spec::screen),
    // let's assume 'cygwin' is xterm-compatible
    ("cygwin", spec::xterm),
];

impl<'a> Spec<'a> {
    pub const empty: Self = Self { keys: [str0_utf8!(""); TB_KEYS_NUM], funcs: [str0_utf8!(""); T_FUNCS_NUM] };
}

pub fn init(buf: &mut [u8]) -> Option<Spec> { unsafe {
    if buf.len() < TI_HEADER_LENGTH << 1 { return None }

    if let None = load_terminfo(buf) { return init_builtin() }

    let hdr = slice::from_raw_parts(buf.as_mut_ptr() as *mut [u8; 2], TI_HEADER_LENGTH);
    let buf = buf.get(((TI_HEADER_LENGTH + u16_le(hdr[3]) as usize) << 1) +
                      (u16_le(hdr[1]) + u16_le(hdr[2]) + 1) as usize & !1 ..)?;
    let hdr_4 = u16_le(hdr[4]) as usize;
    let (str, tab) = buf.try_split_at(hdr_4 << 1)?;
    let str = slice::from_raw_parts(str.as_ptr() as *mut [u8; 2], hdr_4);
    let tab = &tab[0..tab.iter().rposition(|&b| 0 == b)?];

    let mut spec = Spec::empty;
    for i in 0..TB_KEYS_NUM {
        spec.keys[i] = NulStr::new_unchecked(tab.get(u16_le(*str.get(ti_keys[i] as usize)?) as usize)?)
    }
    for i in 0..T_FUNCS_NUM {
        spec.funcs[i] = NulStr::new_unchecked(tab.get(u16_le(*str.get(ti_funcs[i] as usize)?) as usize)?)
    }
    Some(spec)
} }

#[inline(always)]
fn u16_le(bs: [u8; 2]) -> u16 { bs[0] as u16 | (bs[1] as u16) << 8 }

macro_rules! chain {
    [] => (::core::iter::empty());
    [$x0:expr $(, $x:expr)*] => ($x0.into_iter().chain(chain![$($x),*]));
}

fn load_terminfo(buf: &mut [u8]) -> Option<usize> {
    let name = environ.get("TERM".as_bytes())??;

    if let Some(terminfo) = environ.get("TERMINFO".as_bytes()).and_then(|a|a) {
        return try_terminfo_path(buf, name, terminfo.iter().cloned())
    }

    if let Some(home) = environ.get("HOME".as_bytes()).and_then(|a|a) {
        if let Some(n) = try_terminfo_path(buf, name, chain![home.iter().cloned(),
                                                             "/.terminfo".bytes()]) { return Some(n) }
    }

    if let Some(dirs) = environ.get("TERMINFO_DIRS".as_bytes()).and_then(|a|a) {
        for dir in dirs[..].split(|&b| b':' == b) {
            let dir = if 0 == dir.len() { "/usr/share/terminfo".as_bytes() } else { dir };
            if let Some(n) = try_terminfo_path(buf, name, dir.iter().cloned()) { return Some(n) }
        }
    }

    try_terminfo_path(buf, name, "/usr/share/terminfo".bytes())
}

fn try_terminfo_path<Bs: Iterator<Item = u8>>(buf: &mut [u8], name: &Nul<u8>, bs: Bs) -> Option<usize> {
    let n = fill_slice(buf, bs)?;
    try_terminfo_path_helper(buf, name, n)
}

fn try_terminfo_path_helper(buf: &mut [u8], name: &Nul<u8>, mut k: usize) -> Option<usize> {
    buf.get_mut(k..k+3)?.copy_from_slice(&[b'/', unsafe { *name.as_ptr() }, b'/']);
    k += 3;
    for &b in name.iter() {
        *buf.get_mut(k)? = b;
        k += 1;
    }
    *buf.get_mut(k)? = 0;
    let path = unsafe { Nul::new_unchecked(buf.as_ptr()) };

    use unix::file::*;
    let mut file = open_at(None, path, OpenMode::RdOnly, None).ok()?;
    file.try_read_full(buf).ok()
}

fn fill_slice<A, As: Iterator<Item = A>>(tgt: &mut [A], src: As) -> Option<usize> {
    let mut k = 0;
    for a in src {
        *tgt.get_mut(k)? = a;
        k += 1
    }
    Some(k)
}

fn init_builtin() -> Option<Spec<'static>> {
    let term = environ.get("TERM".as_bytes())??;
    for &(name, spec) in terms { if *name.as_bytes() == term[..] { return Some(spec) } }
    for &(name, spec) in terms_compat { if let Some(_) = name.as_bytes().find(&term[..]) { return Some(spec) } }
    None
}

const TI_MAGIC: u16 = 12;
const TI_HEADER_LENGTH: usize = 6;

static ti_funcs: [u16; T_FUNCS_NUM] = [28, 40, 16, 13, 5, 39, 36, 27, 26, 34, 89, 88];
static ti_keys : [u16; TB_KEYS_NUM] = [
	66, 68 /* apparently not a typo; 67 is F10 for whatever reason */, 69,
	70, 71, 72, 73, 74, 75, 67, 216, 217, 77, 59, 76, 164, 82, 81, 87, 61,
	79, 83,
];
