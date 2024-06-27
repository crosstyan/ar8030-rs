// ARTOSYN
// Interestingly, it's registered as "Linux Foundation"
use rusb::{Device, DeviceHandle, UsbContext};
use log::{info, warn};
// more on kv logging
// https://docs.rs/log/latest/log/kv/index.html
const ARTO_RTOS_VID: u16 = 0x1d6b;
const ARTO_RTOS_PID: u16 = 0x8030;

fn print_info<T>(dev: &Device<T>) -> Result<(), anyhow::Error>
where
    T: UsbContext,
{
    let dev_desc = dev.device_descriptor()?;
    // https://libusb.sourceforge.io/api-1.0/
    info!(bus=dev.bus_number(),
          address=dev.address(),
          port=dev.port_number(),
          speed:?=dev.speed(),
          vid=format!("0x{:04x}", dev_desc.vendor_id()),
          pid=format!("0x{:04x}", dev_desc.product_id());
          "device");

    let config_desc = dev.active_config_descriptor()?;
    let ifaces = config_desc.interfaces();
    for iface in ifaces {
        let iface_descs = iface.descriptors();
        for iface_desc in iface_descs {
            info!(iface=iface.number(),
                  class=iface_desc.class_code(),
                  subclass=iface_desc.sub_class_code(),
                  protocol_code=iface_desc.protocol_code(),
                  length=iface_desc.length();
                  "interface");
            let endpoints = iface_desc.endpoint_descriptors();
            for endpoint in endpoints {
                info!(iface=iface.number(),
                      endpoint=endpoint.number(),
                      direction:?=endpoint.direction(),
                      address=endpoint.address(),
                      interval=endpoint.interval(),
                      length=endpoint.length(),
                      max_packet_size=endpoint.max_packet_size();
                      "endpoint");
            }
        }
    }
    let hdl = dev.open()?;
    let manufacturer = hdl.read_manufacturer_string_ascii(&dev_desc).unwrap_or_default();
    let product = hdl.read_product_string_ascii(&dev_desc).unwrap_or_default();
    let serial = hdl.read_serial_number_string_ascii(&dev_desc).unwrap_or_default();
    info!(manufacturer=manufacturer,
          serial=serial,
          product=product;
          "string descriptors");
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    // set the log level to debug if not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
    let devs = rusb::devices()?;
    for dev in devs.iter() {
        let desc = dev.device_descriptor()?;
        let vid = desc.vendor_id();
        let pid = desc.product_id();
        if vid == ARTO_RTOS_VID && pid == ARTO_RTOS_PID {
            print_info(&dev)?;
        }
    }
    return Ok(());
}
