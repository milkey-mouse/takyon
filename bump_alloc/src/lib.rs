#![no_std]
use core::{
    alloc::{GlobalAlloc, Layout},
    // TODO: should be arch::wasm, see https://github.com/Craig-Macomber/lol_alloc/issues/1
    arch::wasm32::memory_grow,
    cell::UnsafeCell,
};

const PAGE_SIZE: usize = 65536;

pub struct BumpAllocator(UnsafeCell<BumpAllocatorInner>);

impl BumpAllocator {
    pub const fn new() -> BumpAllocator {
        Self(UnsafeCell::new(BumpAllocatorInner { ptr: 0, cap: 0 }))
    }
}

// TODO: not really Sync; we rely on wasm being single-threaded. document this
unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self.0.get()).alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

struct BumpAllocatorInner {
    ptr: usize,
    cap: usize,
}

impl BumpAllocatorInner {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let aligned = self.ptr % layout.align();
        if self.ptr != aligned {
            self.ptr = aligned + layout.align();
        }

        let alloc = self.ptr as *mut u8;

        self.ptr += layout.size();
        if self.ptr > self.cap {
            let new_bytes = self.ptr - self.cap;
            let new_pages = (new_bytes + PAGE_SIZE - 1) / PAGE_SIZE;
            memory_grow(0, new_pages);
            self.cap += new_pages * PAGE_SIZE;
        }

        alloc
    }
}
