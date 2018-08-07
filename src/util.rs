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

pub mod slice {
    use core::slice;

    #[inline(always)]
    pub unsafe fn split_at_unchecked<A>(xs: &[A], k: usize) -> (&[A], &[A]) {
        (slice::from_raw_parts(xs.as_ptr(), k),
         slice::from_raw_parts(xs.as_ptr().add(k), xs.len() - k))
    }

    #[inline(always)]
    pub unsafe fn split_at_unchecked_mut<A>(xs: &mut [A], k: usize) -> (&mut [A], &mut [A]) {
        (slice::from_raw_parts_mut(xs.as_mut_ptr(), k),
         slice::from_raw_parts_mut(xs.as_mut_ptr().add(k), xs.len() - k))
    }
}
