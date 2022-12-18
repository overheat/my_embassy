
use defmt::*;
use embassy_net::{Stack};

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<cyw43::NetDevice<'static>>) -> ! {
    stack.run().await
}

