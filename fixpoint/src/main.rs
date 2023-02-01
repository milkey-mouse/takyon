use bump_alloc::BumpAllocator;
use takyon::TachyonicAntitelephone;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

fn main() {
    let antitelephone = TachyonicAntitelephone::new();

    // receive the message from the future
    println!("received '{}' from the future", antitelephone.recv());

    // send "hello world" into the past
    unsafe { antitelephone.send_raw("hello world!") };
}
