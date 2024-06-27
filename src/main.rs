use std::sync::Arc;
use parking_lot::Mutex;
use rusb::{Device, DeviceHandle, UsbContext, DeviceDescriptor, TransferType, Direction, Context, HotplugBuilder, Hotplug};
use log::{info, warn, error, debug, trace};

// more on kv logging
// https://docs.rs/log/latest/log/kv/index.html
// ARTOSYN
// Interestingly, it's registered as "Linux Foundation"
const ARTO_RTOS_VID: u16 = 0x1d6b;
const ARTO_RTOS_PID: u16 = 0x8030;
const BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
struct Endpoint {
    /// config number
    config: u8,
    /// interface number
    iface: u8,
    /// alternative setting number
    setting: u8,
    /// endpoint number
    number: u8,
    /// endpoint address
    address: u8,
    /// endpoint direction
    direction: Direction,
    /// endpoint max packet size
    max_packet_size: u16,
    /// endpoint transfer type
    transfer_type: TransferType,
}

// from https://github.com/a1ien/rusb/blob/master/examples/read_device.rs
fn find_endpoints<T: UsbContext>(
    device: &Device<T>,
) -> Result<Vec<Endpoint>, anyhow::Error> {
    let device_desc = device.device_descriptor()?;
    let mut ret = Vec::<Endpoint>::new();
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    let e = Endpoint {
                        config: config_desc.number(),
                        iface: interface_desc.interface_number(),
                        setting: interface_desc.setting_number(),
                        number: endpoint_desc.number(),
                        address: endpoint_desc.address(),
                        direction: endpoint_desc.direction(),
                        transfer_type: endpoint_desc.transfer_type(),
                        max_packet_size: endpoint_desc.max_packet_size(),
                    };
                    ret.push(e);
                }
            }
        }
    }

    Ok(ret)
}

fn print_info<T: UsbContext>(hdl: &DeviceHandle<T>) -> Result<(), anyhow::Error> {
    let dev = hdl.device();
    let dev_desc = dev.device_descriptor()?;
    let num_configs = dev_desc.num_configurations();
    // https://libusb.sourceforge.io/api-1.0/
    info!(bus=dev.bus_number(),
          address=dev.address(),
          port=dev.port_number(),
          speed:?=dev.speed(),
          num_configs=num_configs,
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
                      transfer_type:?=endpoint.transfer_type(),
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

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<(), anyhow::Error> {
    // handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

/// loop and poll the endpoint
fn poll_endpoint<T: UsbContext>(
    handle: &Mutex<DeviceHandle<T>>,
    endpoint: Endpoint,
) -> Result<(), anyhow::Error> {
    if endpoint.direction != Direction::In {
        warn!(iface=endpoint.iface,
              endpoint=endpoint.number;
              "endpoint is not IN");
        anyhow::bail!("endpoint is not IN");
    }
    let mut g_hdl = handle.lock();
    configure_endpoint(&mut g_hdl, &endpoint)?;
    drop(g_hdl);

    let timeout = std::time::Duration::from_millis(100);
    loop {
        let mut buf = [0u8; BUFFER_SIZE];
        let g_hdl = handle.lock();
        let res = g_hdl.read_bulk(endpoint.address, &mut buf, timeout);
        drop(g_hdl);
        match res {
            Ok(n) => {
                let content = &buf[..n];
                // print as hex
                let hex = content.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("");
                info!(len=n, hex=hex, iface=endpoint.iface, endpoint=endpoint.number; "read");
            }
            Err(e) => {
                if e == rusb::Error::Timeout {
                    debug!(
                        iface=endpoint.iface,
                        endpoint=endpoint.number;
                        "timeout");
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        }
    }
}

/// auto detach kernel driver if supported.
/// return true if the driver is detached, false if not supported, error otherwise
fn auto_detach_kernel_driver<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<bool, anyhow::Error> {
    let res = match handle.set_auto_detach_kernel_driver(true) {
        Ok(_) => Ok(true),
        Err(e) => {
            if e == rusb::Error::NotSupported {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }?;
    Ok(res)
}


struct HotPlugHandler;
impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        info!("device arrived {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        info!("device left {:?}", device);
    }
}

impl Drop for HotPlugHandler {
    fn drop(&mut self) {
        info!("HotPlugHandler dropped");
    }
}

fn main() -> Result<(), anyhow::Error> {
    // set the log level to debug if not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    if !rusb::has_hotplug() {
        warn!("hotplug not supported on this platform");
    }

    let version = rusb::version();
    info!("libusb version={}.{}.{}.{} rc={}", version.major(), version.minor(), version.micro(), version.nano(), version.rc().unwrap_or("None"));

    // or use global context
    let mut ctx = Context::new()?;
    ctx.set_log_level(rusb::LogLevel::Info);
    // https://github.com/a1ien/rusb/blob/master/examples/hotplug.rs

    // TODO: hotplug https://libusb.sourceforge.io/api-1.0/libusb_hotplug.html
    let devs = ctx.devices()?;
    let mut handles: Vec<Arc<Mutex<DeviceHandle<Context>>>> = vec![];
    for dev in devs.iter() {
        let desc = dev.device_descriptor()?;
        let vid = desc.vendor_id();
        let pid = desc.product_id();
        if vid == ARTO_RTOS_VID && pid == ARTO_RTOS_PID {
            let mut hdl = dev.open()?;
            let _ = auto_detach_kernel_driver(&mut hdl)?;
            print_info(&hdl)?;
            handles.push(Arc::new(Mutex::new(hdl)));
            // see following functions in the daemon
            // `start_8030_proc`
            // `set_usb_info`
            // `dev_node_list_init` (start two threads)
            //   - `dev_send_thread`
            //   - `dev_recv_thread`
        }
    }
    for hdl in handles.into_iter() {
        // well let's poll the device
        // https://libusb.sourceforge.io/api-1.0/group__libusb__poll.html
        let g_hdl = hdl.lock();
        let endpoints = find_endpoints(&g_hdl.device()).unwrap();
        drop(g_hdl);

        for ep in endpoints {
            info!("endpoint={:?}", ep);
            if ep.direction == Direction::In {
                let hdl = hdl.clone();
                std::thread::spawn(move || {
                    let res = poll_endpoint(&hdl, ep);
                    if let Err(e) = res {
                        error!("poll error={:?}", e);
                    }
                });
            } else if ep.direction == Direction::Out {}
        }
    }
    // TODO: hotplug
    // see `usbrpc_8030_poll`
    // the original daemon implement use polling based approach
    loop {}
    return Ok(());
}
