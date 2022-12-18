
use     embassy_rp::{
        flash::Flash,
        gpio::{Flex, Level, Output},
        interrupt,
        peripherals::{FLASH, PIN_23, PIN_24, PIN_25, PIN_29, USB},
        usb::Driver,
    };
#[embassy_executor::task]
pub async fn usb_task(u: USB) {
    let usb_irq = interrupt::take!(USBCTRL_IRQ);
    let driver = Driver::new(u, usb_irq);
    // embassy_usb_logger::run!(1024, log::LevelFilter::Debug, driver);
}