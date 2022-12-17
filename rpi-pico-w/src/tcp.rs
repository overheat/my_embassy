use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources, Ipv4Address, Ipv4Cidr};
use embassy_net::tcp::TcpSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_time::{Duration, Timer};
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi::{ExclusiveDevice, SpiBusFlush, SpiBusRead, SpiBusWrite};
use embedded_io::asynch::Write;
use heapless::{String, Vec};
use reqwless::client::{HttpClient, TlsConfig};
use reqwless::request::{ContentType, Method};
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn listen_task(stack: &'static Stack<cyw43::NetDevice<'static>>) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("rxd {:02x}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

