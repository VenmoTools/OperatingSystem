use bitflags::bitflags;
use crate::devices::bus::pic::class::PciClass;
use crate::devices::bus::pic::{PciBaseAddress, CfgSpaceReader};
use byteorder::LittleEndian;
use byteorder::ByteOrder;

use super::data::{DevicesVendorId, DeviceId, Command, PciEventStatus, Revision, Interface, SubClass,
                  CacheLineSize, LatencyTimer, BIST, CardBusCisPtr, SubSystemVendorId, SubSystemId,
                  ExpansionRomBar, CapPointer, InterruptLine, InterruptPin, MinGrant, MaxLatency,
                  PrimaryBusNum, SecondaryBusNum, SubordinateBusNum, SecondaryLatencyTimer, IoBase,
                  IoLimit, SecondaryStatus, MemBase, MemLimit, PrefetchBase, PrefetchLimit,
                  PrefetchLimitUpper, IoBaseUpper, IoLimitUpper, ExpansionRom, BridgeControl};

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    NoDevices,
    UnknownDevices(u8),
}

bitflags! {
    /// Flags found in the status register of a PCI device
    pub struct PciHeaderType: u8 {
        /// A general PCI device (Type 0x01).
        const GENERAL       = 0b00000000;
        /// A PCI-to-PCI bridge device (Type 0x01).
        const PCITOPCI      = 0b00000001;
        /// A PCI-to-PCI bridge device (Type 0x02).
        const CARDBUSBRIDGE = 0b00000010;
        /// A multifunction device.
        const MULTIFUNCTION = 0b01000000;
        /// Mask used for fetching the header type.
        const HEADER_TYPE   = 0b00000011;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PciHeader {
    General(GeneralDevices),
    PciToPci(PciToPci),
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeneralDevices {
    vendor_id: DevicesVendorId,
    device_id: DeviceId,
    command: Command,
    status: PciEventStatus,
    revision: Revision,
    interface: Interface,
    subclass: SubClass,
    class: PciClass,
    cache_line_size: CacheLineSize,
    latency_timer: LatencyTimer,
    header_type: PciHeaderType,
    bist: BIST,
    bars: [PciBaseAddress; 6],
    cardbus_cis_ptr: CardBusCisPtr,
    subsystem_vendor_id: SubSystemVendorId,
    subsystem_id: SubSystemId,
    expansion_rom_bar: ExpansionRomBar,
    cap_pointer: CapPointer,
    interrupt_line: InterruptLine,
    interrupt_pin: InterruptPin,
    min_grant: MinGrant,
    max_latency: MaxLatency,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PciToPci {
    vendor_id: DevicesVendorId,
    device_id: DeviceId,
    command: Command,
    status: PciEventStatus,
    revision: Revision,
    interface: Interface,
    subclass: SubClass,
    class: PciClass,
    cache_line_size: CacheLineSize,
    latency_timer: LatencyTimer,
    header_type: PciHeaderType,
    bist: BIST,
    bars: [PciBaseAddress; 2],
    primary_bus_num: PrimaryBusNum,
    secondary_bus_num: SecondaryBusNum,
    subordinate_bus_num: SubordinateBusNum,
    secondary_latency_timer: SecondaryLatencyTimer,
    io_base: IoBase,
    io_limit: IoLimit,
    secondary_status: SecondaryStatus,
    mem_base: MemBase,
    mem_limit: MemLimit,
    prefetch_base: PrefetchBase,
    prefetch_limit: PrefetchLimit,
    prefetch_base_upper: PrefetchLimitUpper,
    prefetch_limit_upper: PrefetchLimitUpper,
    io_base_upper: IoBaseUpper,
    io_limit_upper: IoLimitUpper,
    cap_pointer: CapPointer,
    expansion_rom: ExpansionRom,
    interrupt_line: InterruptLine,
    interrupt_pin: InterruptPin,
    bridge_control: BridgeControl,
}

impl PciHeader {
    pub fn parser<R: CfgSpaceReader>(reader: R) -> Result<Self> {
        let dev = unsafe { reader.read(0) };
        if dev == 0xffffffff {
            return Err(Error::NoDevices);
        }
        let bytes = unsafe { reader.read_range(0, 16) };
        let vendor_id = DevicesVendorId::new(&bytes[0..2]);
        let device_id = DeviceId::new(&bytes[2..4]);

        let command = Command::new(&bytes[4..6]);
        let status = PciEventStatus::new(&bytes[6..8]);
        let revision = Revision::new(bytes[8]);
        let interface = Interface::new(bytes[9]);
        let subclass = SubClass::new(bytes[10]);
        let class = PciClass::from(bytes[11]);
        let cache_line_size = CacheLineSize::new(bytes[12]);
        let latency_timer = LatencyTimer::new(bytes[13]);
        let header_type = PciHeaderType::from_bits_truncate(bytes[14]);
        let bist = BIST::new(bytes[15]);

        match header_type & PciHeaderType::HEADER_TYPE {
            PciHeaderType::GENERAL => {
                let bytes = unsafe { reader.read_range(16, 48) };
                let bars = [
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[0..4])),
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[4..8])),
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[8..12])),
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[12..16])),
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[16..20])),
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[20..24])),
                ];
                let cardbus_cis_ptr = CardBusCisPtr::new(&bytes[24..28]);
                let subsystem_vendor_id = SubSystemVendorId::new(&bytes[28..30]);
                let subsystem_id = SubSystemId::new(&bytes[30..32]);
                let expansion_rom_bar = ExpansionRomBar::new(&bytes[32..36]);
                let cap_pointer = CapPointer::new(bytes[36]);
                let interrupt_line = InterruptLine::new(bytes[44]);
                let interrupt_pin = InterruptPin::new(bytes[45]);
                let min_grant = MinGrant::new(bytes[46]);
                let max_latency = MaxLatency::new(bytes[47]);
                Ok(PciHeader::General(GeneralDevices {
                    vendor_id,
                    device_id,
                    command,
                    status,
                    revision,
                    interface,
                    subclass,
                    class,
                    cache_line_size,
                    latency_timer,
                    header_type,
                    bist,
                    bars,
                    cardbus_cis_ptr,
                    subsystem_vendor_id,
                    subsystem_id,
                    expansion_rom_bar,
                    cap_pointer,
                    interrupt_line,
                    interrupt_pin,
                    min_grant,
                    max_latency,
                }))
            }
            PciHeaderType::PCITOPCI => {
                let bytes = unsafe { reader.read_range(16, 48) };
                let bars = [
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[0..4])),
                    PciBaseAddress::from(LittleEndian::read_u32(&bytes[4..8])),
                ];
                let primary_bus_num = PrimaryBusNum::new(bytes[8]);
                let secondary_bus_num = SecondaryBusNum::new(bytes[9]);
                let subordinate_bus_num = SubordinateBusNum::new(bytes[10]);
                let secondary_latency_timer = SecondaryLatencyTimer::new(bytes[11]);
                let io_base = IoBase::new(bytes[12]);
                let io_limit = IoLimit::new(bytes[13]);
                let secondary_status = SecondaryStatus::new(&bytes[14..16]);
                let mem_base = MemBase::new(&bytes[16..18]);
                let mem_limit = MemLimit::new(&bytes[18..20]);
                let prefetch_base = PrefetchBase::new(&bytes[20..22]);
                let prefetch_limit = PrefetchLimit::new(&bytes[22..24]);
                let prefetch_base_upper = PrefetchLimitUpper::new(&bytes[24..28]);
                let prefetch_limit_upper = PrefetchLimitUpper::new(&bytes[28..32]);
                let io_base_upper = IoBaseUpper::new(&bytes[32..34]);
                let io_limit_upper = IoLimitUpper::new(&bytes[34..36]);
                let cap_pointer = CapPointer::new(bytes[36]);
                let expansion_rom = ExpansionRom::new(&bytes[40..44]);
                let interrupt_line = InterruptLine::new(bytes[44]);
                let interrupt_pin = InterruptPin::new(bytes[45]);
                let bridge_control = BridgeControl::new(&bytes[46..48]);
                Ok(PciHeader::PciToPci(PciToPci {
                    vendor_id,
                    device_id,
                    command,
                    status,
                    revision,
                    interface,
                    subclass,
                    class,
                    cache_line_size,
                    latency_timer,
                    header_type,
                    bist,
                    bars,
                    primary_bus_num,
                    secondary_bus_num,
                    subordinate_bus_num,
                    secondary_latency_timer,
                    io_base,
                    io_limit,
                    secondary_status,
                    mem_base,
                    mem_limit,
                    prefetch_base,
                    prefetch_limit,
                    prefetch_base_upper,
                    prefetch_limit_upper,
                    io_base_upper,
                    io_limit_upper,
                    cap_pointer,
                    expansion_rom,
                    interrupt_line,
                    interrupt_pin,
                    bridge_control,
                }))
            }
            id => {
                Err(Error::UnknownDevices(id.bits()))
            }
        }
    }
    /// Return the Pci Header Type field.
    pub fn header_type(&self) -> PciHeaderType {
        match self {
            PciHeader::General(dev) => dev.header_type,
            PciHeader::PciToPci(dev) => dev.header_type
        }
    }
    /// Return the Devices Vendor Id field.
    pub fn vendor_id(&self) -> DevicesVendorId {
        match self {
            PciHeader::General(dev) => dev.vendor_id,
            PciHeader::PciToPci(dev) => dev.vendor_id
        }
    }
    /// Return the Pic Class field.
    pub fn class(&self) -> PciClass {
        match self {
            PciHeader::General(dev) => dev.class,
            PciHeader::PciToPci(dev) => dev.class
        }
    }

    /// Return the Device ID field.
    pub fn device_id(&self) -> DeviceId {
        match self {
            PciHeader::General(dev) => dev.device_id,
            PciHeader::PciToPci(dev) => dev.device_id
        }
    }

    /// Return the Revision field.
    pub fn revision(&self) -> Revision {
        match self {
            PciHeader::General(dev) => dev.revision,
            PciHeader::PciToPci(dev) => dev.revision
        }
    }

    /// Return the Interface field.
    pub fn interface(&self) -> Interface {
        match self {
            PciHeader::General(dev) => dev.interface,
            PciHeader::PciToPci(dev) => dev.interface
        }
    }

    /// Return the Subclass field.
    pub fn subclass(&self) -> SubClass {
        match self {
            PciHeader::General(dev) => dev.subclass,
            PciHeader::PciToPci(dev) => dev.subclass
        }
    }

    pub fn bars(&self) -> &[PciBaseAddress] {
        match self {
            PciHeader::General(ref dev) => dev.bars.as_ref(),
            PciHeader::PciToPci(ref dev) => dev.bars.as_ref(),
        }
    }

    /// Return the BAR at the given index.
    ///
    /// # Panics
    /// This function panics if the requested BAR index is beyond the length of the header
    /// types BAR array.
    pub fn get_bar(&self, idx: usize) -> PciBaseAddress {
        match self {
            PciHeader::General(dev) => {
                assert!(idx < 6, "the general PCI device only has 6 BARs");
                dev.bars[idx]
            }
            PciHeader::PciToPci(dev) => {
                assert!(idx < 2, "the general PCI device only has 2 BARs");
                dev.bars[idx]
            }
        }
    }

    /// Return the Interrupt Line field.
    pub fn interrupt_line(&self) -> InterruptLine {
        match self {
            PciHeader::General(dev) => dev.interrupt_line,
            PciHeader::PciToPci(dev) => dev.interrupt_line
        }
    }

    pub fn status(&self) -> PciEventStatus {
        match self {
            PciHeader::General(dev) => dev.status,
            PciHeader::PciToPci(dev) => dev.status
        }
    }

    pub fn cap_pointer(&self) -> CapPointer {
        match self {
            PciHeader::General(dev) => dev.cap_pointer,
            PciHeader::PciToPci(dev) => dev.cap_pointer
        }
    }
}