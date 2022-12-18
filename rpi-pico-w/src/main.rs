#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources, Ipv4Address, Ipv4Cidr};
use embassy_rp::interrupt;
use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::adc::{Adc, Config};
use embassy_rp::usb::Driver;
use embassy_rp::peripherals::{PIN_23, PIN_25};
use embassy_time::{Duration, Timer};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embedded_hal_async::spi::{ExclusiveDevice};
use heapless::{Vec};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod get_temperature;
mod logger;
mod out;
mod tcp;
mod udp;
mod v4;
mod wifi;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}


static CHANNEL: StaticCell<Channel<NoopRawMutex, f32, 1>> = StaticCell::new();



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let channel = CHANNEL.init(Channel::new());

    // Include the WiFi firmware and Country Locale Matrix (CLM) blobs.
    // let fw = include_bytes!("../firmware/43439A0.bin");
    // let clm = include_bytes!("../firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs-cli download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs-cli download 43439A0.clm_blob --format bin --chip RP2040 --base-address 0x10140000
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let clk = Output::new(p.PIN_29, Level::Low);
    let mut dio = Flex::new(p.PIN_24);
    dio.set_low();
    dio.set_as_output();

    let bus = wifi::Spi { clk, dio };
    let spi = ExclusiveDevice::new(bus, cs);

    let state = singleton!(cyw43::State::new());
    let (mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    spawner.spawn(wifi::wifi_task(runner)).unwrap();

    let net_device = control.init(clm).await;

    //control.join_open(env!("WIFI_NETWORK")).await;
    control.join_wpa2(env!("WIFI_NETWORK"), env!("WIFI_PASSWORD")).await;

    let mut dns_servers = Vec::new();
    dns_servers.push(Ipv4Address::new(1, 1, 1, 1)); // Cloudflare DNS
    dns_servers.push(Ipv4Address::new(8, 8, 8, 8)); // Google DNS

    // let config = embassy_net::ConfigStrategy::Dhcp;
    let config = embassy_net::ConfigStrategy::Static(embassy_net::Config {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 31, 111), 24),
        dns_servers,
        gateway: Some(Ipv4Address::new(192, 168, 31, 1)),
    });

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let stack = &*singleton!(Stack::new(
        net_device,
        config,
        singleton!(StackResources::<1, 2, 8>::new()),
        seed
    ));

    spawner.spawn(v4::net_task(stack)).unwrap();

    // And now we can use it!
    info!("Application initialized.");

    Timer::after(Duration::from_secs(10)).await;

    // spawner.spawn(logger::usb_task(p.USB)).unwrap();
    spawner.spawn(get_temperature::get(p.ADC, channel.sender())).unwrap();
    spawner.spawn(out::pub_task(stack, seed, channel.receiver())).unwrap();
    // spawner.spawn(tcp::listen_task(stack)).unwrap();
    // spawner.spawn(udp::listen_task(stack)).unwrap();


}