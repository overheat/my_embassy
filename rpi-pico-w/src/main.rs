#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use defmt::*;
use embassy_executor::Spawner;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel};
use embassy_net::{Stack};
use {defmt_rtt as _, panic_probe as _};
use static_cell::StaticCell;

mod get_temperature;
mod logger;
mod out;
mod tcp;
mod udp;
mod v4;
mod wifi;
pub enum Services {
    Temperature,
    DNS,
    NTP,
    Other,
}
static CHANNEL: StaticCell<Channel<NoopRawMutex, &'static Stack<cyw43::NetDevice<'static>>, 1>> = StaticCell::new();
static IN_CHANNEL: StaticCell<Channel<NoopRawMutex, Services, 1>> = StaticCell::new();
static OUT_CHANNEL: StaticCell<Channel<NoopRawMutex, f32, 1>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let wifi_stack = CHANNEL.init(Channel::new());
    let _in = IN_CHANNEL.init(Channel::new());
    let out = OUT_CHANNEL.init(Channel::new());

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.
    spawner.spawn(wifi::init(p.PIN_23, p.PIN_25, p.PIN_29, p.PIN_24, seed, wifi_stack.sender())).unwrap();

    let stack = wifi_stack.recv().await;

    // And now we can use it!
    info!("Application initialized.");
    spawner.spawn(v4::net_task(stack)).unwrap();

    // Timer::after(Duration::from_secs(10)).await;

    // spawner.spawn(logger::usb_task(p.USB)).unwrap();
    spawner.spawn(tcp::listen_task(stack, _in.sender())).unwrap();
    spawner.spawn(get_temperature::get(p.ADC, _in.receiver(), out.sender())).unwrap();
    spawner.spawn(out::pub_task(stack, seed, out.receiver())).unwrap();
    // spawner.spawn(udp::listen_task(stack)).unwrap();


}