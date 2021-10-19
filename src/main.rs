//!
//! # Memory Management
//! 
//! Pool of Segments: it's a variation of the bitmaps memory management system.
//! 
//! Flat model. Divide memory in buckets. Each bucket is a range of memory that contains memory segments of the same size.<br>
//! We have multiple buckets with different segment sizes to allocate memory with different requierements, because each Alloc returns exactly one segment.<br>
//! We have a fixed number of buckets ordered from smaller segment size to bigger.<br>
//! At start, we create a struct for each bucket, that contains:
//! 
//! - Stack. We put in the stack the starting address of each segment.
//! - Segment size (in byte).
//! - Bucket size (total number of segments).
//! - Counter with the current number of free segments.
//! 
//! Once an alloc happens, we check the size requested and we select the bucket with the closest segment size.
//! We pop an address from the stack and decrease the counter.
//! If no segments available in the bucklet, we try with the next bucket size, and so on.
//! 
//! When a free happens, we just push the segment address into the bucket (each segment has a header with a pointer to the bucket struct it belongs to), and increase the counter.
//! 
//! Advantages:
//! 
//! - Predictable and fast Alloc and Free operation times, O(1) complexity.
//! - No need for long mutex cycles that lock other tasks, only simple atomic PopAddress and PushAddress operation that are very short.
//! 
//! Disadvantages:
//! - Is not possible to guarantee contiguous segments when we alloc, and then we have less flexibility (resize operation is not feasible).
//! - More affected by fragmentation, more likely to get nothing from Alloc than other classic allocation methods (like linked lists).
//! 
//! Drawbacks can be mitigated by chosing convenient segment and bucket sizes.
//! 
//! # Device Model
//! 
//! We need a `Device` trait, that controls each device, with a missage passing interface (`DeviceMessage` trait and `DeviceResponse` trait) that is something like ioctl.
//! 
//! Then we need a `Driver` trait, that is the real interface the apps will use.
//! 
//! Drivers talk to Devices. Drivers are arch independant, Devices are arch dependant. One Driver can be used to talk to multiple Devices, for example: the ConsoleDriver could use the VideoDevice + KeyboardDevice or the SerialDevice.
//! 
//! We could have a pluggable interface for Inputs and Outputs, so we can configure drivers to work with any combination of devices, without having to be aware of which device it is. Maybe a series of traits: `InputFlow`, `OutputFlow`, `BidiFlow`.<br>
//! We could use it to create complex flow chains: a ConsoleDriver that reads input from a KeyboardDevice, but sends the output to a TcpDriver that in turn sends it to the SlipDriver that sends it to the SerialDevice.
//! 


#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt;

#[macro_use]
mod console;
use console::*;

mod counter_future;
use counter_future::*;

use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicI32;

mod devices;
use devices::{
    InputFlow, OutputFlow,
    console::{
        ConCmd, ConsoleDevice, CON_DEVICE, AnsiColor
    }
};

mod sys;
use sys::{
    KMutex, KLock
};

static TEST : KMutex<TestStruct> = KMutex::new(TestStruct { count: 0, buf: [0;32] });

struct TestStruct {
    count: usize,
    buf: [u8 ; 32]
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_title("-- Rust Kernel Test --");
    print_count();
    println!("MT my_val {}", MT.my_val.load(Ordering::SeqCst));
    MT.my_val.store(100, Ordering::SeqCst);
    println!("After change, MT my_val {}", MT.my_val.load(Ordering::SeqCst));

    CON_DEVICE.write_cmd(
        ConCmd::Print(10, 24, AnsiColor::BrightRed, AnsiColor::BrightCyan),
        "Final thing!".as_bytes()
    ).unwrap_or_default();

    println!("TEST lock count = {}", TEST.lock().count);

    loop {}
}

// Experimental console usage
fn print_title(msg: &str) {
    let center = 40 - msg.len() / 2;

    let console = unsafe { CONSOLE_WRITER.console() };
    console << (center, 11, msg);

    let console = Console::new(ColorScheme::default());
    &console << (center, 13, msg);
}

// Regular console usage
fn print_count() {
    for i in 0..10 {
        println!("Counter {}", i);
    }
    println!();
}