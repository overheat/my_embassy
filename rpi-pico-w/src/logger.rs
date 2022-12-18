
use embassy_rp::{
        interrupt,
        peripherals::{USB},
        usb::Driver,
    };
#[embassy_executor::task]
pub async fn usb_task(u: USB) {
    let irq = interrupt::take!(USBCTRL_IRQ);
    let _driver = Driver::new(u, irq);
    // embassy_usb_logger::run!(1024, log::LevelFilter::Debug, driver);
}