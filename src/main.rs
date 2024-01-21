#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rustos::println;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, RustOS!");
    loop {}
}
