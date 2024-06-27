// ARTOSYN
// Interestingly, it's registered as "Linux Foundation"
const ARTO_RTOS_VID: u16 = 0x1d6b;
const ARTO_RTOS_PID: u16 = 0x8030;

fn main() -> Result<(), anyhow::Error> {
    let devs = rusb::devices()?;
    for dev in devs.iter() {
        let desc = dev.device_descriptor()?;
        let vid = desc.vendor_id();
        let pid = desc.product_id();
        if vid == ARTO_RTOS_VID && pid == ARTO_RTOS_PID {
            println!("AR8030: Bus {:03} Device {:03} ID {:04x}:{:04x}",
                     dev.bus_number(), dev.address(),
                     desc.vendor_id(), desc.product_id());
            let desc = dev.device_descriptor()?;
            println!("  Class: {:03} {:03} {:03}",
                     desc.class_code(), desc.sub_class_code(), desc.protocol_code());
            let dev_hdl = dev.open()?;
        }
    }
    return Ok(());
}
