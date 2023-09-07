use std::{str, sync::Mutex};

use dotenv_codegen::dotenv;
use embedded_svc::{http::Method, ws::FrameType};
use esp_idf_hal::{
    delay,
    i2c::{config::Config, I2cDriver},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_svc::{
    errors::EspIOError, eventloop::EspSystemEventLoop, http::server::EspHttpServer,
    nvs::EspDefaultNvsPartition,
};
use esp_idf_sys::{self as _, EspError}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

pub mod wifi;

pub mod icm42670p;
use icm42670p::{DeviceAddr, ICM42670P};

use crate::wifi::wifi;

static INDEX_HTML: &str = include_str!("index.html");
const STACK_SIZE: usize = 10240;

fn create_server() -> Result<EspHttpServer, EspIOError> {
    let config = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    EspHttpServer::new(&config)
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    // Take control of the peripherals available in the ESP32-C3.
    let peripherals = Peripherals::take().unwrap();

    // Take ownership of the System Event loop.
    let sysloop = EspSystemEventLoop::take().unwrap();

    // Take ownership of the Non-Volatile Storage.
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Take ownership of the I2C0 module and configure it.
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio10,
        peripherals.pins.gpio8,
        &Config::default().baudrate(Hertz(400_000)),
    )
    .unwrap();

    // Create instance of the ICM42670P sensor pass I2C port used.
    let mut imu = ICM42670P::new(i2c, DeviceAddr::AD0).unwrap();

    // Set IMU's accelerometer state from Idle to Low-Noise mode.
    imu.set_accel_in_low_noise_mode().unwrap();

    // Create WiFi instance.
    let _wifi = wifi(
        dotenv!("WIFI_SSID"),
        dotenv!("WIFI_PASS"),
        peripherals.modem,
        sysloop,
        nvs,
    );

    // Create Web Server.
    let mut server = create_server().unwrap();

    // Render HTML on Web Server.
    server
        .fn_handler("/", Method::Get, |req| {
            req.into_ok_response()?.write(INDEX_HTML.as_bytes())?;
            Ok(())
        })
        .unwrap();

    let imu = Mutex::new(imu);

    server
        .ws_handler("/ws/imu", move |ws| {
            let mut imu = imu.lock().unwrap();

            if ws.is_new() {
                info!("New WebSocket session");
                ws.send(
                    FrameType::Text(false),
                    "[ESP-RS]: Connected to the ESP-RS WebSocket server.".as_bytes(),
                )?;
            }

            // Get length of the WebSocket message received.
            let (_frame_type, len) = match ws.recv(&mut []) {
                Ok(frame) => frame,
                Err(e) => return Err(e),
            };

            // Verify if the message is too long to store.
            if len > 64 {
                ws.send(
                    FrameType::Text(false),
                    "[ESP-RS]: Request too big.".as_bytes(),
                )?;
                ws.send(FrameType::Close, &[])?;
            }

            // Retrieve WebSocket message.
            let mut buf = [0; 64];
            ws.recv(buf.as_mut())?;

            // Convert WebSocket message from bytes to string.
            let Ok(recv_string) = str::from_utf8(&buf[..len]) else {
                ws.send(FrameType::Text(false), "[ESP-RS]: UTF-8 Error.".as_bytes())?;
                return Ok(());
            };

            match recv_string.trim_end_matches(char::from(0)) {
                "GET IMU" => {
                    ws.send(
                        FrameType::Text(false),
                        format!(
                            "ICM42670P Accelerometer => X: {}, Y: {}, Z: {}",
                            imu.read_accel_x().unwrap(),
                            imu.read_accel_y().unwrap(),
                            imu.read_accel_z().unwrap()
                        )
                        .as_ref(),
                    )?;
                }
                _ => {
                    info!(
                        "WebSocket message received from the client: '{}'",
                        recv_string
                    );
                    ws.send(FrameType::Text(false), "[ESP-RS]: Unknown data.".as_bytes())?;
                }
            }

            Ok::<(), EspError>(())
        })
        .unwrap();

    loop {
        // let mut imu = imu.lock().unwrap();

        // let accel_x = imu.read_accel_x().unwrap();
        // let accel_y = imu.read_accel_y().unwrap();
        // let accel_z = imu.read_accel_z().unwrap();

        // println!(
        //     "ICM42670P Accelerometer => X: {}, Y: {}, Z: {}",
        //     accel_x, accel_y, accel_z
        // );

        delay::FreeRtos::delay_ms(250);
    }
}
