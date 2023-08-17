use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

use log;

pub fn wifi(
    ssid: &str,
    pass: &str,
    modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Box<EspWifi<'static>> {
    // Verify that the SSID and password strings are populated.
    if ssid.is_empty() {
        log::error!("Missing SSID!");
    }
    if pass.is_empty() {
        log::warn!("No password provided.")
    }

    // Create WiFi driver instance.
    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs)).unwrap();

    // Configure Wifi by providing the credentials.
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.into(),
        password: pass.into(),
        ..Default::default()
    }))
    .unwrap();

    // Connect to WiFi.
    wifi.start().unwrap();
    wifi.connect().unwrap();
    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
    }
    log::info!("Should be connected to WiFi now...");

    // Return WiFi instance.
    Box::new(wifi)
}
