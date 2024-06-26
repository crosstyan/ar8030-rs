fn main() -> Result<(), anyhow::Error> {
    let devs = rusb::devices()?;
    for dev in devs.iter() {
        let desc = dev.device_descriptor()?;
        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                 dev.bus_number(), dev.address(),
                 desc.vendor_id(), desc.product_id());
    }
    return Ok(());
}
