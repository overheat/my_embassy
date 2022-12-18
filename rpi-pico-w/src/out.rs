use defmt::*;

use embassy_net::{Stack};
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_time::{Duration, Timer};
use heapless::{String};
use reqwless::client::{HttpClient, TlsConfig};
use reqwless::request::{ContentType, Method};


/// HTTP endpoint hostname
const HOSTNAME: &str = "http.sandbox.drogue.cloud";

/// HTTP endpoint port
const PORT: &str = "443";

/// HTTP username
const USERNAME: &str = "device1@embassy-app";

/// HTTP password
const PASSWORD: &str = "hey-rodney";


#[path = "../common/dns.rs"]
mod dns;
use dns::*;

#[path = "../common/temperature.rs"]
mod temperature;
use temperature::*;


#[embassy_executor::task]
pub async fn pub_task(stack: &'static Stack<cyw43::NetDevice<'static>>, seed: u64, temperature: f32) -> ! {
    static CLIENT_STATE: TcpClientState<1, 1024, 1024> = TcpClientState::new();
    let client = TcpClient::new(&stack, &CLIENT_STATE);

    let url: String<128> = String::from("https://http.sandbox.drogue.cloud/v1/pico");
    // write!(url, "https://{}:{}/v1/pico", HOSTNAME, PORT).unwrap();

    let mut tls = [0; 16384];
    let mut client = HttpClient::new_with_tls(&client, &DNS, TlsConfig::new(seed as u64, &mut tls));

    loop {
        Timer::after(Duration::from_secs(5)).await;
        let sensor_data = TemperatureData {
            geoloc: None,
            temp: Some(temperature),
            hum: None,
        };

        info!("Reporting sensor data: {:?}", sensor_data.temp);

        let tx: String<128> = serde_json_core::ser::to_string(&sensor_data).unwrap();
        let mut rx_buf = [0; 1024];
        let response = client
            .request(Method::POST, &url)
            .await
            .unwrap()
            .basic_auth(USERNAME.trim_end(), PASSWORD.trim_end())
            .body(tx.as_bytes())
            .content_type(ContentType::ApplicationJson)
            .send(&mut rx_buf[..])
            .await;

        match response {
            Ok(response) => {
                // info!("Response status: {:?}", response.status);
                if let Some(payload) = response.body {
                    let _s = core::str::from_utf8(payload).unwrap();
                }
            }
            Err(_e) => {
                // warn!("Error doing HTTP request: {:?}", e);
                warn!("Error doing HTTP request: ");
            }
        }
        info!("Telemetry reported successfully");

    }
}