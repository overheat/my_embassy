use core::convert::Infallible;

use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi::{SpiBusFlush, SpiBusRead, SpiBusWrite, ExclusiveDevice};
use embassy_net::{Stack, StackResources, Ipv4Address, Ipv4Cidr};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Sender};
use static_cell::StaticCell;
use heapless::{Vec};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}
#[embassy_executor::task]
pub async fn init(pwr: PIN_23,cs: PIN_25,clk: PIN_29,dio: PIN_24, seed: u64,
                    sender: Sender<'static, NoopRawMutex, &'static Stack<cyw43::NetDevice<'static>>, 1>) {
    // Include the WiFi firmware and Country Locale Matrix (CLM) blobs.
    // let fw = include_bytes!("../firmware/43439A0.bin");
    // let clm = include_bytes!("../firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs-cli download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs-cli download 43439A0.clm_blob --format bin --chip RP2040 --base-address 0x10140000
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(pwr, Level::Low);
    let cs = Output::new(cs, Level::High);
    let clk = Output::new(clk, Level::Low);
    let mut dio = Flex::new(dio);
    dio.set_low();
    dio.set_as_output();

    let bus = Spi { clk, dio };
    let spi = ExclusiveDevice::new(bus, cs);

    let state = singleton!(cyw43::State::new());
    let (control, runner) = cyw43::new(state, pwr, spi, fw).await;

    // spawner.spawn(wifi_task(runner)).unwrap();
    runner.run().await;

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

    // Init network stack
    let stack = &*singleton!(Stack::new(
        net_device,
        config,
        singleton!(StackResources::<1, 2, 8>::new()),
        seed
    ));

    sender.send(stack).await;
}

// #[embassy_executor::task]
// pub async fn wifi_task(
//     runner: cyw43::Runner<'static, Output<'static, PIN_23>, ExclusiveDevice<Spi, Output<'static, PIN_25>>>,
// ) -> ! {
//     runner.run().await
// }

pub struct Spi {
    /// SPI clock
    pub clk: Output<'static, PIN_29>,

    /// 4 signals, all in one!!
    /// - SPI MISO
    /// - SPI MOSI
    /// - IRQ
    /// - strap to set to gSPI mode on boot.
    pub dio: Flex<'static, PIN_24>,
}

impl ErrorType for Spi {
    type Error = Infallible;
}

impl SpiBusFlush for Spi {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SpiBusRead<u32> for Spi {
    async fn read(&mut self, words: &mut [u32]) -> Result<(), Self::Error> {
        self.dio.set_as_input();
        for word in words {
            let mut w = 0;
            for _ in 0..32 {
                w = w << 1;

                // rising edge, sample data
                if self.dio.is_high() {
                    w |= 0x01;
                }
                self.clk.set_high();

                // falling edge
                self.clk.set_low();
            }
            *word = w
        }

        Ok(())
    }
}

impl SpiBusWrite<u32> for Spi {
    async fn write(&mut self, words: &[u32]) -> Result<(), Self::Error> {
        self.dio.set_as_output();
        for word in words {
            let mut word = *word;
            for _ in 0..32 {
                // falling edge, setup data
                self.clk.set_low();
                if word & 0x8000_0000 == 0 {
                    self.dio.set_low();
                } else {
                    self.dio.set_high();
                }

                // rising edge
                self.clk.set_high();

                word = word << 1;
            }
        }
        self.clk.set_low();

        self.dio.set_as_input();
        Ok(())
    }
}
