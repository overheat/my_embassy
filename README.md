# myEmbassy

## Description
    every 5 seconds temperature sensor trigger out async function through channel. And this out function send message to cloud.
## Running the example

- `cargo install probe-run`
- `cd examples/rpi-pico-w`
- `WIFI_NETWORK=MyWifiNetwork WIFI_PASSWORD=MyWifiPassword cargo run --release`

## Receive cloud message
```
websocat wss://ws-integration.sandbox.drogue.cloud/embassy-app -H="Authorization: Bearer $(drg whoami -t)"
```
