extern crate util;
pub use self::util::*;

use core::mem;

#[inline(always)]
pub unsafe fn copy_from_slice<A: Copy>(tgt: *mut A, src: &[A]) {
    tgt.copy_from_nonoverlapping(src.as_ptr(), src.len())
}

#[inline(always)]
pub unsafe fn copy_from_ptr<A: Copy>(tgt: &mut [A], src: *const A) {
    src.copy_to_nonoverlapping(tgt.as_mut_ptr(), tgt.len())
}

#[inline(always)]
pub fn ptr_diff<A>(q: *mut A, p: *mut A) -> usize { (q as usize - p as usize)/mem::size_of::<A>() }

#[inline(always)]
pub const unsafe fn uninitialized<A>() -> A {
    union U<A> { u: (), v: A };
    U { u: () }.v
}
