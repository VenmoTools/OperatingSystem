use crate::bits::BitOpt;
use bitflags::bitflags;

macro_rules! impl_structure_u8 {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq,Debug)]
        pub struct $name{
            pub inner: u8,
        }

        impl $name {
            pub fn new(inner: u8) -> Self{
                Self{
                    inner
                }
            }
        }
    };
}

macro_rules! impl_structure_u16 {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq,Debug)]
        pub struct $name{
            pub inner: u16,
        }

        impl $name {
            pub fn new(buf: &[u8]) -> Self{
                use byteorder::ByteOrder;
                Self{
                    inner: ::byteorder::LittleEndian::read_u16(buf)
                }
            }
        }
    };
}

macro_rules! impl_structure_u32 {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq,Debug)]
        pub struct $name{
            pub inner: u32,
        }

        impl $name {
            pub fn new(buf: &[u8]) -> Self{
                use byteorder::ByteOrder;
                Self{
                    inner: ::byteorder::LittleEndian::read_u32(buf)
                }
            }
        }
    };
}

macro_rules! rw_bit {
    ($set_name:ident,$read_name:ident,$bit:expr) => {

        pub fn $set_name(&mut self, enable: bool) {
            self.inner.set_bit($bit, enable);
        }

        pub fn $read_name(&self) -> bool{
            self.inner.get_bit($bit)
        }
    };
}

macro_rules! rw_range {
    ($set_name:ident,$read_name:ident,$range:expr,$ty:ty) => {
        pub fn $set_name(&mut self, data: $ty) {
            self.inner.set_bits($range, data);
        }

        pub fn $read_name(&self) -> $ty{
            self.inner.get_bits($range)
        }
    };
}

impl_structure_u8!(Revision);
impl_structure_u8!(Interface);
impl_structure_u8!(CacheLineSize);
impl_structure_u8!(LatencyTimer);
impl_structure_u8!(BIST); // built-in self test
impl_structure_u8!(CapPointer);
impl_structure_u8!(InterruptLine);
impl_structure_u8!(InterruptPin);
impl_structure_u8!(MaxLatency);
impl_structure_u8!(PrimaryBusNum);
impl_structure_u8!(SecondaryBusNum);
impl_structure_u8!(SubordinateBusNum);
impl_structure_u8!(SecondaryLatencyTimer);
impl_structure_u8!(IoBase);
impl_structure_u8!(IoLimit);
impl_structure_u8!(SubClass);
impl_structure_u8!(MinGrant);

impl_structure_u16!(DevicesVendorId);
impl_structure_u16!(DeviceId);
impl_structure_u16!(Command);

impl Command {
    ///  If set to 1 the assertion of the devices INTx# signal is disabled; otherwise, assertion of the signal is enabled.
    rw_bit!(set_interrupt,interrupt,10);

    ///  If set to 1 indicates a device is allowed to generate fast back-to-back transactions; otherwise, fast back-to-back transactions are only allowed to the same agent.
    rw_bit!(set_fast_back_to_back,fast_back_to_back,9);

    ///  If set to 1 the SERR# driver is enabled; otherwise, the driver is disabled.
    rw_bit!(set_enable_serr,enable_serr,8);

    ///  If set to 1 the device will take its normal action when a parity error is detected;
    /// otherwise, when an error is detected, the device will set bit 15 of the Status register (Detected Parity Error Status Bit),
    /// but will not assert the PERR# (Parity Error) pin and will continue operation as normal.
    rw_bit!(set_parity_error_resp,parity_error_resp,6);

    /// If set to 1 the device does not respond to palette register writes and will snoop the data;
    /// otherwise, the device will trate palette write accesses like all other accesses.
    rw_bit!(set_vga_palette_snoop,vga_palette_snoopv,5);

    /// If set to 1 the device can generate the Memory Write and Invalidate command;
    /// otherwise, the Memory Write command must be used.
    rw_bit!(set_memory_write_and_invalidate_enable,memory_write_and_invalidate_enable,4);

    ///  If set to 1 the device can monitor Special Cycle operations; otherwise, the device will ignore them.
    rw_bit!(set_special_cycles,special_cycles,3);

    ///  If set to 1 the device can monitor Special Cycle operations; otherwise, the device will ignore them.
    rw_bit!(set_bus_master,bus_master,2);

    /// If set to 1 the device can respond to Memory Space accesses; otherwise, the device's response is disabled.
    rw_bit!(set_memory_space,memory_space,1);

    ///  If set to 1 the device can respond to I/O Space accesses; otherwise, the device's response is disabled.
    rw_bit!(set_io_space,io_space,1);
}

impl_structure_u16!(PciEventStatus);

impl PciEventStatus{

    // This bit will be set to 1 whenever the device detects a parity error, even if parity error handling is disabled.
    rw_bit!(set_detected_parity_error,detected_parity_error,15);

    /// This bit will be set to 1 whenever the device asserts SERR#.
    rw_bit!(set_signaled_system_error,signaled_system_error,14);

    /// This bit will be set to 1, by a master device, whenever its transaction (except for Special Cycle transactions) is terminated with Master-Abort.
    rw_bit!(set_received_master_abort,received_master_abort,13);

    ///  This bit will be set to 1, by a master device, whenever its transaction is terminated with Target-Abort.
    rw_bit!(set_received_target_abort,received_target_abort,12);

    ///  This bit will be set to 1 whenever a target device terminates a transaction with Target-Abort.
    rw_bit!(set_signaled_target_abort,signaled_target_abort,11);

    /// Read only bits that represent the slowest time that a device will assert DEVSEL# for any bus command except Configuration Space read and writes.
    /// Where a value of 0x00 represents fast timing, a value of 0x01 represents medium timing, and a value of 0x02 represents slow timing.
    rw_range!(set_devsel_timing,devsel_timing,9..11,u16);

    /// This bit is only set when the following conditions are met. The bus agent asserted PERR# on a read or observed an assertion of PERR# on a write,
    /// the agent setting the bit acted as the bus master for the operation in which the error occurred,
    /// and bit 6 of the Command register (Parity Error Response bit) is set to 1.
    rw_bit!(set_master_data_parity_error,master_data_parity_error,8);


    ///  If set to 1 the device can accept fast back-to-back transactions that are not from the same agent;
    /// otherwise, transactions can only be accepted from the same agent.
    rw_bit!(set_fast_back_to_back_capable,fast_back_to_back_capable,7);

    // Bit 6 - As of revision 3.0 of the PCI local bus specification this bit is reserved.
    // In revision 2.1 of the specification this bit was used to indicate whether or not a device supported User Definable Features.

    ///  If set to 1 the device is capable of running at 66 MHz; otherwise, the device runs at 33 MHz.
    rw_bit!(set_capable66_mhz,capable66_mhz,5);

    /// If set to 1 the device implements the pointer for a New Capabilities Linked list at offset 0x34;
    /// otherwise, the linked list is not available.
    rw_bit!(set_capabilities_list,capabilities_list,4);

    /// Represents the state of the device's INTx# signal.
    /// If set to 1 and bit 10 of the Command register (Interrupt Disable bit) is set to 0 the signal will be asserted;
    /// otherwise, the signal will be ignored.
    rw_bit!(set_interrupt_status,interrupt_status,3);
}

impl_structure_u16!(SubSystemVendorId);
impl_structure_u16!(SubSystemId);
impl_structure_u16!(SecondaryStatus);
impl_structure_u16!(MemBase);
impl_structure_u16!(MemLimit);
impl_structure_u16!(PrefetchBase);
impl_structure_u16!(PrefetchLimit);
impl_structure_u16!(IoBaseUpper);
impl_structure_u16!(IoLimitUpper);
impl_structure_u16!(BridgeControl);

impl_structure_u32!(PrefetchBaseUpper);
impl_structure_u32!(PrefetchLimitUpper);
impl_structure_u32!(ExpansionRom);
impl_structure_u32!(CardBusCisPtr);
impl_structure_u32!(ExpansionRomBar);

bitflags!{
    pub struct CommandType: u16{
        /// The Interrupt Acknowledge command is a read implicitly addressed to the system interrupt
        /// controller
        const INTERRUPT_ACKNOWLEDGE = 0000;
        /// The Special Cycle command provides a simple message broadcast mechanism on PCI. It is
        /// designed to be used as an alternative to physical signals when sideband communication is
        /// necessary
        const SPECIAL_CYCLE  = 0001 ;
        /// The I/O Read command is used to read data from an agent mapped in I/O Address Space.
        /// AD[31::00] provide a byte address. All 32 bits must be decoded. The byte enables indicate
        /// the size of the transfer and must be consistent with the byte address
        const IO_READ  = 0010 ;
        /// The I/O Write command is used to write data to an agent mapped in I/O Address Space.
        /// All 32 bits must be decoded. The byte enables indicate the size of the transfer and must be
        /// consistent with the byte address.
        const IO_WRITE  = 0011 ;
        /// The Memory Read command is used to read data from an agent mapped in the Memory
        /// Address Space. The target is free to do an anticipatory read for this command only if it can
        /// guarantee that such a read will have no side effects
        const MEMORY_READ  = 0110 ;
        /// The Memory Write command is used to write data to an agent mapped in the Memory
        /// Address Space. When the target returns "ready," it has assumed responsibility for the
        /// coherency (which includes ordering) of the subject data
        const MEMORY_WRITE  = 0111 ;
        /// The Configuration Read command is used to read the Configuration Space of each agent. An
        /// agent is selected during a configuration access when its IDSEL signal is asserted and AD[1::0]
        /// are 00.
        const CONFIGURATION_READ  = 1010 ;
        /// The Configuration Write command is used to transfer data to the Configuration Space of each
        /// agent. Addressing for configuration write transactions is the same as for configuration read
        /// transactions
        const CONFIGURATION_WRITE  = 1011 ;
        /// The Memory Read Multiple command is semantically identical to the Memory Read command
        /// except that it additionally indicates that the master may intend to fetch more than one
        /// cacheline before disconnecting.
        const MEMORY_READ_MULTIPLE  = 1100 ;
        /// The Dual Address Cycle (DAC) command is used to transfer a 64-bit address to devices that
        /// support 64-bit addressing when the address is not in the low 4-GB address space. Targets
        /// that support only 32-bit addresses must treat this command as reserved and not respond to
        /// the current transaction in any way.
        const DUAL_ADDRESS_CYCLE  = 1101 ;
        /// The Memory Read Line command is semantically identical to the Memory Read command
        /// except that it additionally indicates that the master intends to fetch a complete cacheline.
        const MEMORY_READ_LINE  = 1110 ;
        /// The Memory Write and Invalidate command is semantically identical to the Memory Write
        /// command except that it additionally guarantees a minimum transfer of one complete
        /// cacheline; i.e., the master intends to write all bytes within the addressed cacheline in a single
        /// PCI transaction unless interrupted by the target.
        /// Note: All byte enables must be asserted during each data phase for this command.
        const MEMORY_WRITE_AND_INVALIDATE = 1111 ;
    }
}