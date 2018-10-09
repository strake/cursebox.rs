use core::num::NonZeroUsize;

static utf8_length: [L; 256] = [
  L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,
  L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,
  L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,
  L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,
  L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,
  L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,L1,
  L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,L2,
  L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L3,L4,L4,L4,L4,L4,L4,L4,L4,L5,L5,L5,L5,L6,L6,L1,L1,
];

static utf8_mask: [u8; 7] = [0, 0x7F, 0x1F, 0x0F, 0x07, 0x03, 0x01];

#[derive(Clone, Copy)]
#[repr(u8)]
enum L {
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
    L6 = 6,
}
use self::L::*;

pub fn decode(bs: &[u8]) -> Option<(u32, NonZeroUsize)> {
    let bs_l = bs.len();
    let (&b0, bs) = bs.split_first()?;
    let l = utf8_length[b0 as usize] as usize;
    if l > bs_l { return None }
    let l = NonZeroUsize::new(l)?;
    let mut x = (b0 & utf8_mask[l.get()]) as u32;
    for b in bs[0..l.get()].iter().cloned() {
        x <<= 6;
        x |= b as u32 & 0x3F;
    }
    Some((x, l))
}
