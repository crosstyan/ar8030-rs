# USB Complete

Copy and paste from the book [USB Complete](https://www.amazon.com/USB-Complete-Developers-Guide-Guides/dp/1931448280) by Jan Axelson.

## USB basics

> Every USB communication (with one exception in USB 3.1) is between a host and
a device. The host manages traffic on the bus, and the device *responds* to
communications from the host. An *endpoint* is a device buffer that stores
received data or data to transmit. Each endpoint address has a number, a
*direction*, and a maximum number of data bytes the endpoint can send or receive
in a transaction.

> Each USB transfer consists of one or more transactions that can carry data to
or from an endpoint.
> In addition to data, each data packet contains error-checking bits and a
Packet ID (PID) with a data-sequencing value. Many transac- tions also have a
handshake packet where the receiver of the data reports success or failure of
the transaction.

> The transactions contain similar addressing, error-checking, and
data-sequencing val- ues along with the data.

- control
- bulk
- interrupt
- isochronous

### Endpoints

The source and sink of data.

An endpoint address consists of an endpoint number and direction. The number is
a value in the range 0–15. *The direction is defined from the host’s perspective*

- an IN endpoint provides data to send *to the host* 
- an OUT endpoint stores data received *from the host*

> Every device must have endpoint zero configured as a control endpoint. Additional control endpoints offer no improvement in performance and thus are rare.

In addition to endpoint zero, a full-speed or high-speed device can have up to
30 additional endpoint addresses (1–15, IN and OUT). 

A single endpoint number can support both IN and OUT endpoint addresses. 

### Transaction Types

Every USB 2.0 transaction begins with a packet that contains an endpoint number
and a code that indicates the direction of data flow and whether the transaction
is initiating a control transfer.

In every USB 2.0 transaction, the host sends an addressing triple that consists
of a device address, an endpoint number, and endpoint direction. 

On receiving an OUT or Setup packet, the endpoint stores the data that follows
the packet, and the device hardware typically triggers an interrupt. 

### Pipes

> connecting endpoints to the host.

Before data can transfer, the host and device must establish a pipe. A pipe is
an association between a device’s endpoint and the host controller’s software.

### Transfers Types

In a control transfer, the host sends a defined request to the device. On
device attachment, the host uses control transfers to request a series of data
structures called descriptors from the device. The *descriptors* provide
information about the device’s *capabilities* and help the host decide what
driver to assign to the device. 

A class specification or vendor can also define requests.

Control transfers have up to three stages: Setup, Data (optional), and Status. 
The other transfer types don’t have defined stages. Instead, higher-level
software defines *how to interpret the raw data*.

- *Bulk transfers* are the fastest on an otherwise idle bus but have no guaranteed timing. Printers and drives (disk) use bulk transfers. 
- *Interrupt transfers* have guaranteed maximum latency, or time between transaction
attempts.  Mice and keyboards use interrupt transfers. 
- *Isochronous transfers* have guaranteed timing but no error correcting. Streaming
audio and video use isochronous transfers.

USB communications fall into two general categories: 

- communications that help to identify and configure the device 
- communications that carry out the device’s purpose.

Every device has a unique address assigned by the host, and all data travels to or from the host.

### Initiating a Transfer

> For receiving data from a device, some drivers request the host controller to
poll an endpoint at intervals, while other drivers don’t initiate communications
unless an application has requested data from the device.


## libusb

- [FAQ](https://github.com/libusb/libusb/wiki/FAQ)
- [gousb](https://github.com/google/gousb) (it's from Google!)
- [rusb](https://github.com/a1ien/rusb)
