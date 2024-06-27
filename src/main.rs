// ARTOSYN
// Interestingly, it's registered as "Linux Foundation"
use rusb::{Device, DeviceHandle, UsbContext};
use log::{info, warn};
const ARTO_RTOS_VID: u16 = 0x1d6b;
const ARTO_RTOS_PID: u16 = 0x8030;

fn handle_ar8030<T>(dev: &Device<T>) -> Result<(), anyhow::Error>
where
    T: UsbContext,
{
    let desc = dev.device_descriptor()?;
    info!(bus=dev.bus_number(),
          address=dev.address(),
          vid=format!("0x{:04x}", desc.vendor_id()),
          pid=format!("0x{:04x}", desc.product_id());
          "AR8030 info");
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
            handle_ar8030(&dev)?;
        }
    }
    return Ok(());
}
