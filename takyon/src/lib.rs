#![feature(layout_for_ptr, ptr_metadata)]
use std::{
    alloc::{alloc, Layout},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr,
};

#[link(wasm_import_module = "takyon")]
extern "C" {
    fn at_open() -> u64;
    fn at_send(channel: u64, ptr: u32, len: u32);
    fn at_recv(channel: u64, ptr: u32, len: u32);
}

#[inline]
unsafe fn send<T: ?Sized>(channel: u64, ptr: *const T, len: usize) {
    at_send(channel, ptr as *const () as _, len as _);
}

#[inline]
unsafe fn recv<T: ?Sized>(channel: u64, ptr: *mut T, len: usize) {
    at_recv(channel, ptr as *mut () as _, len as _);
}

#[derive(Clone)]
#[repr(transparent)]
pub struct TachyonicAntitelephone<T: ?Sized> {
    channel: u64,
    phantom: PhantomData<T>,
}

// TODO
/*impl<T: Borrow<TachyonicAntitelephone>> Read for T {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {}
}*/

impl<T: ?Sized + 'static> TachyonicAntitelephone<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            channel: unsafe { at_open() },
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn send_raw(&self, msg: *const T) {
        let layout = Layout::for_value_raw(msg);
        let (ptr, metadata) = msg.to_raw_parts();
        send(self.channel, &layout as *const _, mem::size_of_val(&layout));
        send(
            self.channel,
            &metadata as *const _,
            mem::size_of_val(&metadata),
        );
        send(self.channel, ptr, layout.size());
        // TODO: this will fail terrifically if T references other data!
    }

    #[inline]
    pub unsafe fn recv_raw(&self) -> *mut T {
        let layout = {
            let mut ptr = MaybeUninit::uninit();
            recv(self.channel, ptr.as_mut_ptr(), mem::size_of_val(&ptr));
            ptr.assume_init()
        };
        let metadata = {
            let mut ptr = MaybeUninit::uninit();
            recv(self.channel, ptr.as_mut_ptr(), mem::size_of_val(&ptr));
            ptr.assume_init()
        };
        let ptr = alloc(layout) as *mut ();
        recv(self.channel, ptr, layout.size());

        ptr::from_raw_parts_mut(ptr, metadata)
    }

    #[inline]
    pub fn send(&self, msg: Box<T>) {
        unsafe { self.send_raw(Box::into_raw(msg)) }
    }

    #[inline]
    pub fn recv(&self) -> Box<T> {
        unsafe { Box::from_raw(self.recv_raw()) }
    }
}
