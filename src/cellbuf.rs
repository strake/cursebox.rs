use core::{cmp, fmt, marker::PhantomData, slice};
use loca::Alloc;
use ptr::Unique;

use {Attr, Cell};

pub struct CellBuf<A: Alloc> {
    width: usize,
    height: usize,
    cells: Unique<Cell>,
    alloc: A,
}

impl<A: Alloc> fmt::Debug for CellBuf<A> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut dl = fmt.debug_list();
        let mut ptr = self.cells.as_ptr().as_ptr();
        for _ in 0..self.height { unsafe {
            dl.entry(&slice::from_raw_parts(ptr, self.width));
            ptr = ptr.add(self.width);
        } }
        dl.finish()
    }
}

impl<A: Alloc> CellBuf<A> {
    pub const fn new_in(alloc: A) -> Self { Self { width: 0, height: 0, cells: Unique::empty(), alloc } }

    #[inline] pub fn width(&self) -> usize { self.width }
    #[inline] pub fn height(&self) -> usize { self.height }

    fn init(&mut self, width: usize, height: usize) -> Result<(), ::loca::AllocErr> {
        self.cells = if 0 == width || 0 == height { Unique::empty() }
                     else { self.alloc.alloc_array(2 * width * height)?.0 };
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) -> Result<(), ::loca::AllocErr> { unsafe {
        let (oldw, oldh) = (self.width, self.height);
        if (width, height) == (oldw, oldh) { return Ok(()) }
        let oldcells = self.cells;

        self.init(width, height)?;
        {
            let (front, back) = self.cells_mut();
            for x in &mut [front, back] { x.clear(Attr::Default, Attr::Default) }
        }

        let minw = cmp::min(oldw, width);
        let minh = cmp::min(oldh, height);

        for i in 0..2*minh {
            let src = oldcells.as_ptr().as_ptr().add(i*oldw);
            let dst = self.cell_ptr().add(i*width);
            dst.copy_from_nonoverlapping(src, minw);
        }

        if oldw != 0 && oldh != 0 { self.alloc.dealloc_array(oldcells, 2 * oldw * oldh); }
        Ok(())
    } }

    #[inline]
    pub fn cell_ptr(&self) -> *mut Cell { self.cells.as_ptr().as_ptr() }

    #[inline]
    pub fn cells_mut(&mut self) -> (CellsMut, CellsMut) {
        let (width, height, cells) = (self.width, self.height, self.cells.as_ptr().as_ptr());
        (CellsMut { width, height, cells, phantom: PhantomData },
         CellsMut { width, height, cells: cells.wrapping_add(width * height), phantom: PhantomData })
    }

    pub unsafe fn blit_from_raw(&mut self, mut src: *const Cell,
                                x: usize, y: usize, w: usize, h: usize) {
        if x+w > self.width || y+h > self.height { return }

        let mut dst = self.cells_mut().1.at_unchecked_mut(x, y) as *mut Cell;
        for _ in 0..h {
            dst.copy_from_nonoverlapping(src, w);
            dst = dst.add(self.width);
            src = src.add(w);
        }
    }
}

impl<A: Alloc> Drop for CellBuf<A> {
    #[inline]
    fn drop(&mut self) { unsafe {
        if self.width != 0 && self.height != 0 {
            self.alloc.dealloc_array(self.cells, 2 * self.width * self.height);
        }
    } }
}

#[derive(Debug)]
pub struct CellsMut<'a> {
    width: usize, height: usize,
    cells: *mut Cell,
    phantom: PhantomData<&'a mut Cell>,
}

impl<'a> CellsMut<'a> {
    pub fn clear(&mut self, fg: Attr, bg: Attr) { unsafe {
        for i in 0..self.width * self.height {
            *self.cells.add(i) = Cell { ch: ' ' as _, fg, bg }
        }
    } }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> Option<&'a Cell> {
        if x > self.width || y > self.height { None }
        else { Some(unsafe { self.at_unchecked(x, y) }) }
    }

    #[inline]
    pub fn at_mut(&mut self, x: usize, y: usize) -> Option<&'a mut Cell> {
        if x > self.width || y > self.height { None }
        else { Some(unsafe { self.at_unchecked_mut(x, y) }) }
    }

    #[inline]
    pub unsafe fn at_unchecked(&self, x: usize, y: usize) -> &'a Cell {
        &*self.cells.add(y * self.width + x)
    }

    #[inline]
    pub unsafe fn at_unchecked_mut(&mut self, x: usize, y: usize) -> &'a mut Cell {
        &mut *self.cells.add(y * self.width + x)
    }
}
