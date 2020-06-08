use crate::devices::bus::pic::data::SubClass;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PciClass {
    Legacy,
    MassStorageController,
    NetworkController,
    DisplayController,
    MultimediaController,
    MemoryController,
    BridgeDevice,
    SimpleCommunicationController,
    BaseSystemPeripheral,
    InputDeviceController,
    DockingStation,
    Processor,
    SerialBusController,
    WirelessController,
    IntelligentController,
    SatelliteCommunicationController,
    EncryptionController,
    SignalProcessingController,
    Reserved(u8),
    Unknown,
}

impl PciClass {
    pub fn subclass(&self, sub: SubClass) -> PciSubClass {
        use self::PciClass::*;
        use self::PciSubClass::*;
        // yeah.. that is ugly
        match self {
            Legacy => match sub.inner {
                0x00 => NonVGACompatibleDevices,
                0x01 => VGACompatibleDevices,
                other => OtherUnclassifiedDevices(other),
            },
            MassStorageController => match sub.inner {
                0x00 => SCSIBusController,
                0x01 => IDEController,
                0x02 => FloppyDiskController,
                0x03 => IPIBusController,
                0x04 => RAIDController,
                0x05 => ATAController,
                0x06 => SerialATA,
                0x07 => SerialAttachedSCSI,
                0x08 => NonVolatileMemoryController,
                other => OtherMassStorageController(other),
            }
            NetworkController => match sub.inner {
                0x00 => EthernetController,
                0x01 => TokenRingController,
                0x02 => FDDIController,
                0x03 => ATMController,
                0x04 => ISDNController,
                0x05 => WorldFipController,
                0x06 => PICMG214MultiComputing,
                0x07 => InfinibandController,
                0x08 => FabricController,
                other => OtherNetworkController(other),
            },
            DisplayController => match sub.inner {
                0x00 => VGACompatibleController,
                0x01 => XGAController,
                0x02 => Controller3D,
                other => OtherDisplayController(other)
            },
            MultimediaController => match sub.inner {
                0x00 => MultimediaVideoController,
                0x01 => MultimediaAudioController,
                0x02 => ComputerTelephonyDevice,
                0x03 => AudioDevice,
                other => OtherMultimediaController(other)
            },
            MemoryController => match sub.inner {
                0x00 => RAMController,
                0x01 => FlashController,
                other => OtherMemoryController(other),
            },
            BridgeDevice => match sub.inner {
                0x00 => HostBridge,
                0x01 => ISABridge,
                0x02 => EISABridge,
                0x03 => MCABridge,
                0x04 => PCIToPCIBridge,
                0x05 => PCMCIABridge,
                0x06 => NuBusBridge,
                0x07 => CardBusBridge,
                0x08 => RACEwayBridge,
                0x09 => PCIToPCIBridge,
                0x0A => InfiniBandToPCIHostBridge,
                other => OtherBridgeDevice(other)
            },
            SimpleCommunicationController => match sub.inner {
                0x00 => SerialController,
                0x01 => ParallelController,
                0x02 => MultiportSerialController,
                0x03 => Modem,
                0x04 => GPIBController,
                0x05 => SmartCard,
                other => OtherSimpleCommunicationController(other)
            },
            BaseSystemPeripheral => match sub.inner {
                0x00 => PIC,
                0x01 => DMAController,
                0x02 => Timer,
                0x03 => RTCController,
                0x04 => PCIHotPlugController,
                0x05 => SDHostcontroller,
                0x06 => IOMMU,
                other => OtherSystemPeripheral(other)
            }
            InputDeviceController => match sub.inner {
                0x00 => KeyboardController,
                0x01 => DigitizerPen,
                0x02 => MouseController,
                0x03 => ScannerController,
                0x04 => GameportController,
                other => OtherInputDeviceController(other),
            }
            DockingStation => match sub.inner {
                0x00 => Generic,
                other => OtherDockingStation(other)
            }
            Processor => match sub.inner {
                0x00 =>I386,
                0x01 =>I486,
                0x02 =>Pentium,
                0x03 =>PentiumPro,
                0x10 =>Alpha,
                0x20 =>PowerPC,
                0x30 =>MIPS,
                0x40 =>CoProcessor,
                other => OtherProcessor(other)
            }
            SerialBusController => match sub.inner{
                0x00 => FireWireController,  // (IEEE 1394)
                0x01 => ACCESSBus,
                0x02 => SSA,
                0x03 => USBController,
                0x04 => FibreChannel,
                0x05 => SMBus,
                0x06 => InfiniBand 	,
                0x07 => IPMIInterface,
                0x08 => SERCOSInterface,// (IEC 61491),
                0x09 => CANbus,
                other => OtherSerialBusController(other),
            }
            WirelessController => match sub.inner{
                0x00 => IRDACompatibleController,
                0x01 => ConsumerIRController,
                0x10 => RFController,
                0x11 => BluetoothController,
                0x12 => BroadbandController,
                0x20 => EthernetController8021a,
                0x21 => EthernetController8021b,
                other => OtherWirelessController(other),
            }
            IntelligentController =>I20,

            SatelliteCommunicationController => match sub.inner{
                0x00 => SatelliteTVController ,
                0x01 => SatelliteAudioController ,
                0x02=> SatelliteVoiceController ,
                0x03 => SatelliteDataController,
                other => UnknownSatelliteCommunicationController(other)
            }
            EncryptionController => match sub.inner{
                0x00 => NetworkandComputingEncrpytionDecryption ,
                0x10 => EntertainmentEncryptionDecryption,
                other => OtherEncryptionDecryption(other),
            }
            SignalProcessingController => match sub.inner{
                0x00 => DPIOModules,
                0x01 => PerformanceCounters,
                0x10 => CommunicationSynchronizer,
                0x20 => SignalProcessingManagement,
                other => OtherSignalProcessingController(other),
            },
            PciClass::Reserved(res) => PciSubClass::Reserved(*res),
            _ => NoSubClass,
        }
    }
}

impl From<u8> for PciClass {
    fn from(class: u8) -> PciClass {
        match class {
            0x00 => PciClass::Legacy,
            0x01 => PciClass::MassStorageController,
            0x02 => PciClass::NetworkController,
            0x03 => PciClass::DisplayController,
            0x04 => PciClass::MultimediaController,
            0x05 => PciClass::MemoryController,
            0x06 => PciClass::BridgeDevice,
            0x07 => PciClass::SimpleCommunicationController,
            0x08 => PciClass::BaseSystemPeripheral,
            0x09 => PciClass::InputDeviceController,
            0x0A => PciClass::DockingStation,
            0x0B => PciClass::Processor,
            0x0C => PciClass::SerialBusController,
            0x0D => PciClass::WirelessController,
            0x0E => PciClass::IntelligentController,
            0x0F => PciClass::SatelliteCommunicationController,
            0x10 => PciClass::EncryptionController,
            0x11 => PciClass::SignalProcessingController,
            0xFF => PciClass::Unknown,
            reserved => PciClass::Reserved(reserved)
        }
    }
}

impl Into<u8> for PciClass {
    fn into(self) -> u8 {
        match self {
            PciClass::Legacy => 0x00,
            PciClass::MassStorageController => 0x01,
            PciClass::NetworkController => 0x02,
            PciClass::DisplayController => 0x03,
            PciClass::MultimediaController => 0x04,
            PciClass::MemoryController => 0x05,
            PciClass::BridgeDevice => 0x06,
            PciClass::SimpleCommunicationController => 0x07,
            PciClass::BaseSystemPeripheral => 0x08,
            PciClass::InputDeviceController => 0x09,
            PciClass::DockingStation => 0x0A,
            PciClass::Processor => 0x0B,
            PciClass::SerialBusController => 0x0C,
            PciClass::WirelessController => 0x0D,
            PciClass::IntelligentController => 0x0E,
            PciClass::SatelliteCommunicationController => 0x0F,
            PciClass::EncryptionController => 0x10,
            PciClass::SignalProcessingController => 0x11,
            PciClass::Unknown => 0xFF,
            PciClass::Reserved(reserved) => reserved
        }
    }
}

pub enum PciSubClass {
    // class code 0x00 Unclassified
    NonVGACompatibleDevices,
    VGACompatibleDevices,
    OtherUnclassifiedDevices(u8),
    // class code 0x01 Mass Storage Controller v
    SCSIBusController,
    IDEController,
    FloppyDiskController,
    IPIBusController,
    RAIDController,
    ATAController,
    SerialATA,
    SerialAttachedSCSI,
    NonVolatileMemoryController,
    OtherMassStorageController(u8),
    // class code 0x02  Network Controller
    EthernetController,
    TokenRingController,
    FDDIController,
    ATMController,
    ISDNController,
    WorldFipController,
    PICMG214MultiComputing,
    InfinibandController,
    FabricController,
    OtherNetworkController(u8),
    // class code 0x03 Display Controller
    VGACompatibleController,
    XGAController,
    Controller3D,
    // (Not VGA-Compatible),
    OtherDisplayController(u8),
    // class code 0x04 Multimedia Controller
    MultimediaVideoController,
    MultimediaAudioController,
    ComputerTelephonyDevice,
    AudioDevice,
    OtherMultimediaController(u8),
    // class code 0x05 Memory Controller
    RAMController,
    FlashController,
    OtherMemoryController(u8),
    // class code 0x06 Bridge Device
    HostBridge,
    ISABridge,
    EISABridge,
    MCABridge,
    PCIToPCIBridge,
    PCMCIABridge,
    NuBusBridge,
    CardBusBridge,
    RACEwayBridge,
    InfiniBandToPCIHostBridge,
    OtherBridgeDevice(u8),
    // class code 0x07 Simple Communication Controller
    SerialController,
    ParallelController,
    MultiportSerialController,
    Modem,
    //IEEE 488.1/2
    GPIBController,
    SmartCard,
    OtherSimpleCommunicationController(u8),
    // class code 0x08 Base System Peripheral
    PIC,
    DMAController,
    Timer,
    RTCController,
    PCIHotPlugController,
    SDHostcontroller,
    IOMMU,
    OtherSystemPeripheral(u8),
    // class code 0x09 Input Device Controller
    KeyboardController,
    DigitizerPen,
    MouseController,
    ScannerController,
    GameportController,
    Extended,
    OtherInputDeviceController(u8),
    // class code 0x0a Docking Station
    Generic,
    OtherDockingStation(u8),
    // class code 0x0b Processor
    I386,
    I486,
    Pentium,
    PentiumPro,
    Alpha,
    PowerPC,
    MIPS,
    CoProcessor,
    OtherProcessor(u8),
    // class code 0x0c Serial Bus Controller
    FireWireController,  // (IEEE 1394)
    ACCESSBus,
    SSA,
    USBController,
    FibreChannel,
    SMBus,
    InfiniBand 	,
    IPMIInterface,
    SERCOSInterface,// (IEC 61491),
    CANbus,
    OtherSerialBusController(u8),
    // class code 0x0D Wireless Controller
    IRDACompatibleController,
    ConsumerIRController,
    RFController,
    BluetoothController,
    BroadbandController,
    EthernetController8021a,
    EthernetController8021b,
    OtherWirelessController(u8),
    // class code 0x0E  Intelligent Controller
    I20,
    // class code 0x0F  Satellite Communication Controller
    SatelliteTVController ,
    SatelliteAudioController ,
    SatelliteVoiceController ,
    SatelliteDataController,
    UnknownSatelliteCommunicationController(u8),
    // class code 0x10  Encryption Controller
    NetworkandComputingEncrpytionDecryption ,
    EntertainmentEncryptionDecryption,
    OtherEncryptionDecryption(u8),
    // class code 0x11  Signal Processing Controller
    DPIOModules,
    PerformanceCounters,
    CommunicationSynchronizer,
    SignalProcessingManagement,
    OtherSignalProcessingController(u8),
    // class code 0x12  Processing Accelerator
    // class code 0x13  Non-Essential Instrumentation
    // class code 0xFF  Unassigned Class (Vendor specific)
    // class code 0x40  Co-Processor
    NoSubClass,
    // class code 0x41  0xFE (Reserved)
    // class code 0x14  0x3F (Reserved)
    Reserved(u8),

}

impl PciSubClass {
    //todo: programming interface god...give me a break
}