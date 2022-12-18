use defmt::*;
use embassy_rp::adc::{Adc, Config};
use embassy_rp::interrupt;
use embassy_rp::peripherals::ADC;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
pub async fn get(
    a: ADC,
    receiver: Receiver<'static, NoopRawMutex, crate::Services, 1>,
    sender: Sender<'static, NoopRawMutex, f32, 1>,
) -> ! {
    let irq = interrupt::take!(ADC_IRQ_FIFO);
    let mut adc = Adc::new(a, irq, Config::default());

    loop {
        receiver.recv().await;
        let temp = adc.read_temperature().await;
        // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
        let temp: f32 = 27.0 - (temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
        info!("Temp: {} degrees", temp);
        sender.send(temp).await;
        // Timer::after(Duration::from_secs(5)).await;
    }
}
