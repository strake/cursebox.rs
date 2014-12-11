use core::num::NonZeroUsize;

use ringbuffer::Ringbuffer;

pub(crate) const TB_KEYS_NUM: usize = 22;

static event_keys: [Key; TB_KEYS_NUM] = { use self::Key::*; [
    F(1), F(2), F(3), F(4), F(5), F(6), F(7), F(8), F(9), F(10), F(11), F(12),
    Insert, Delete, Home, End, PgUp, PgDn, Up, Down, Left, Right,
] };

fn parse_escape_seq(buf: &[u8], keys: [&::nul::Nul<u8>; TB_KEYS_NUM]) -> Option<(Mod, Key, NonZeroUsize)> {
    for i in 0..TB_KEYS_NUM {
        let key = &keys[i][..];
        if let (Some(n), true) = (NonZeroUsize::new(key.len()), buf.starts_with(key)) {
            return Some((Mod::empty(), event_keys[i], n))
        }
    }
    None
}

const BUFFER_SIZE_MAX: usize = 16;

pub(crate) fn extract_event(inbuf: &mut Ringbuffer, mode: Mode, keys: [&::nul::Nul<u8>; TB_KEYS_NUM]) -> Option<(Mod, Key)> {
    let mut buf: [u8; BUFFER_SIZE_MAX] = unsafe { ::core::mem::uninitialized() };
    let nbytes = ::core::cmp::min(inbuf.data_size(), buf.len());
    if 0 == nbytes { return None }

    inbuf.read(&mut buf[0..nbytes]);
    if 0x1B == buf[0] {
        if let Some((mod_, key, n)) = parse_escape_seq(&buf, keys) {
            inbuf.skip(n.get());
            return Some((mod_, key))
        }

        // it's not escape sequence, so it's ALT or ESC; check mode
        inbuf.skip(1);
        return Some(match mode {
            Mode::Esc => (Mod::empty(), Key::Char('\x1B')),
            Mode::Alt => {
                let (mod_, key) = extract_event(inbuf, mode, keys)?;
                (mod_ | Mod::Alt, key)
            },
        })
    }

    // utf8
    if let Some((x, n)) = ::utf8::decode(&buf[0..nbytes])
               .and_then(|(x, n)| ::core::char::from_u32(x).map(|x| (x, n))) {
        inbuf.skip(n);
        return Some((Mod::empty(), Key::Char(x)))
    }

    None
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Key {
    Tab,
    Enter,
    Esc,
    Backspace,
    Right,
    Left,
    Up,
    Down,
    Delete,
    Insert,
    Home,
    End,
    PgUp,
    PgDn,
    Char(char),
    F(u16),
}

impl Key {
    pub const fn Ctrl(b: u8) -> Self { Key::Char((b & !0x60) as _) }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Event {
    Key(Mod, Key),
    Resize(u32, u32),
}

bitflags! {
    pub struct Mod: u16 {
        const Alt  = 1 << 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode { Esc, Alt }
