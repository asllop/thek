use core::{
    marker::PhantomData
};

use crate::{
    sys::{
        KMutex, KLock, Void
    }
};

use super::super::super::Device;

/// Screen text device.
/// 
/// Can't be directly instantiated.
pub struct ScreenTextDevice(Void);

impl Device<'_> for ScreenTextDevice {
    fn lock() ->  KLock<'static, Self> {
        SCREEN_DEVICE.acquire()
    }

    fn force_unlock() {
        SCREEN_DEVICE.reset();
    }
}

/// Public screen device static instance.
static SCREEN_DEVICE : KMutex<ScreenTextDevice> = KMutex::new(ScreenTextDevice(PhantomData));

//TODO
/*
Create a dynamic interface to register devices and std out/in.
It must be flexible enough to hold different kind of devicec:
- Output (console, serial port, etc).
- Input (keybord, mouse, serial port, etc).
- Network.
- Disk/storage devices.
- Graphics?
*/