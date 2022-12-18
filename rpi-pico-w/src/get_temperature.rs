use defmt::*;
use embassy_rp::peripherals::ADC;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::adc::{Adc, Config};
use embassy_rp::interrupt;
use embassy_rp::Peripheral;

#[embassy_executor::task]
pub async fn get(a: ADC) -> ! {     
    let adc_irq = interrupt::take!(ADC_IRQ_FIFO);
    let mut adc = Adc::new(a, adc_irq, Config::default());

    loop {
        let temp = adc.read_temperature().await;
        // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
        let temp: f32 = 27.0 - (temp as f32 * 3.3 / 4096.0 -0.706)/0.001721;
        info!("Temp: {} degrees", temp);
        Timer::after(Duration::from_secs(3)).await;
    } 
}