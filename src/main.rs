use dotenv_codegen::dotenv;
use esp_idf_hal::{
    delay,
    i2c::{config::Config, I2cDriver},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

pub mod wifi;

pub mod icm42670p;
use icm42670p::{DeviceAddr, ICM42670P};

use crate::wifi::wifi;

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

    loop {
        let accel_x = imu.read_accel_x().unwrap();
        let accel_y = imu.read_accel_y().unwrap();
        let accel_z = imu.read_accel_z().unwrap();

        println!(
            "ICM42670P Accelerometer => X: {}, Y: {}, Z: {}",
            accel_x, accel_y, accel_z
        );

        delay::FreeRtos::delay_ms(250);
    }
}
