use util::*;

#[derive(Debug)]
pub struct Ringbuffer<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) begin: *mut u8,
    pub(crate) end: *mut u8,
}

impl<'a> Ringbuffer<'a> {
    pub fn free_space(&self) -> usize {
        if self.begin.is_null() && self.end.is_null() { self.buf.len() }
        else if self.end > self.begin { self.buf.len() - ptr_diff(self.end, self.begin) - 1 }
        else { ptr_diff(self.begin, self.end) - 1 }
    }

    pub fn data_size(&self) -> usize {
        if self.begin.is_null() && self.end.is_null() { 0 }
        else if self.end >= self.begin { ptr_diff(self.end, self.begin) + 1 }
        else { self.buf.len() - ptr_diff(self.begin, self.end) + 1 }
    }

    #[inline(always)]
    pub fn clear(&mut self) { self.begin = 0 as _; self.end = 0 as _; }

    #[inline(always)]
    fn buf_end_ptr(&self) -> *mut u8 { self.buf.as_ptr().wrapping_add(self.buf.len()) as _ }

    #[inline(always)]
    fn to_end_from(&self, ptr: *mut u8) -> usize { ptr_diff(self.buf_end_ptr(), ptr) }

    pub fn push(&mut self, bs: &[u8]) { unsafe {
        if self.free_space() < bs.len() { return }
        if self.begin.is_null() && self.end.is_null() {
            copy_from_slice(self.buf.as_mut_ptr(), bs);
            self.begin = self.buf.as_mut_ptr();
            self.end   = self.begin.add((bs.len() - 1) as _);
            return
        }

        self.end = self.end.add(1);
        if self.begin < self.end && self.to_end_from(self.begin) < bs.len() {
            // make a cut
            let (xs, ys) = slice::split_at_unchecked(bs, self.to_end_from(self.end));
            copy_from_slice(self.end, xs);
            copy_from_slice(self.buf.as_mut_ptr(), ys);
            self.end = self.buf.as_mut_ptr().add(ys.len() - 1);
        } else {
            // fits with no cut
            copy_from_slice(self.end, bs);
            self.end = self.end.add(bs.len() - 1);
        }
    } }

    pub unsafe fn pop_raw(&mut self, ptr: *mut u8, mut size: usize) {
        if self.data_size() < size { return }
        let need_clear = self.data_size() == size;

        if self.begin < self.end || self.to_end_from(self.begin) >= size {
            if !ptr.is_null() { ptr.copy_from_nonoverlapping(self.begin, size) }
            self.begin = self.begin.add(size);
        } else {
            let s = self.to_end_from(self.begin);
            if !ptr.is_null() { ptr.copy_from_nonoverlapping(self.begin, s) }
            size -= s;
            if !ptr.is_null() { ptr.add(s).copy_from_nonoverlapping(self.buf.as_ptr(), size) }
            self.begin = self.buf.as_mut_ptr().add(size);
        }

        if need_clear { self.clear() }
    }

    #[inline]
    pub fn skip(&mut self, n: usize) { unsafe { self.pop_raw(0 as _, n) } }

    #[inline]
    pub fn pop(&mut self, bs: &mut [u8]) { unsafe { self.pop_raw(bs.as_mut_ptr(), bs.len()) } }

    #[inline]
    pub fn read(&self, bs: &mut [u8]) { unsafe {
        let mut other = ::core::ptr::read(self);
        other.pop_raw(bs.as_mut_ptr(), bs.len());
        ::core::mem::forget(other);
    } }

    pub fn push_from_file(&mut self, file: &mut ::unix::file::File) -> Result<usize, ::unix::err::OsErr> { unsafe {
        use core::slice;
        use io::Read;

        if self.begin.is_null() && self.end.is_null() {
            let n = file.read(self.buf)?;
            self.begin = self.buf.as_mut_ptr();
            self.end   = self.begin.add((n - 1) as _);
            return Ok(n)
        }

        let end = self.end.add(1);
        let n = file.readv(&mut [slice::from_raw_parts_mut(end, self.to_end_from(end)),
                                 slice::from_raw_parts_mut(self.buf.as_mut_ptr(),
                                                           ptr_diff(self.begin,
                                                                    self.buf.as_mut_ptr()))])?;
        self.end = self.end.add(n);
        if self.end >= self.buf_end_ptr() { self.end = self.end.sub(self.buf.len()) }
        Ok(n)
    } }
}
