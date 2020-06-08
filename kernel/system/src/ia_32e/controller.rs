use crate::ia_32e::xapic::xApic;
use crate::ia_32e::x2apic::local_apic::LocalApic;
use crate::ia_32e::cpu::ChainedPics;
use crate::ia_32e::x2apic::io_apic::{IoApic, IrqMode};
use crate::bits::IrqFlags;
use crate::ia_32e::x2apic::register::{TimerMode, TimerDivide, IpiAllShorthand};
use crate::ia_32e::ApicInfo;
use crate::alloc::string::String;
use core::marker::PhantomData;

pub trait ControllerType {
    const DISPLAY_STR: &'static str;
}


pub struct XPAIC(xApic);

pub struct X2APIC(LocalApic,IoApic);

pub struct PIC(ChainedPics);

impl ControllerType for XPAIC {
    const DISPLAY_STR: &'static str = "xapic";
}

impl ControllerType for X2APIC {
    const DISPLAY_STR: &'static str = "x2apic";
}

impl ControllerType for PIC {
    const DISPLAY_STR: &'static str = "8259A";
}

macro_rules! set_attr {
    ($name:ident,$attr:ident,$args:ty) => {
        pub fn $name(&mut self, args: $args) {
            self.$attr = Some(args)
        }
    };
}
pub struct ProgrammableController<T: ControllerType> {
    local_apic: Option<LocalApic>,
    io_apic: Option<IoApic>,
    xapic: Option<xApic>,
    pic: Option<ChainedPics>,
    _mark: PhantomData<T>,
}

impl<T: ControllerType> ProgrammableController<T> {
    pub fn name() -> &'static str {
        T::DISPLAY_STR
    }

    pub const fn empty() -> Self {
        Self {
            local_apic: None,
            io_apic: None,
            xapic: None,
            pic: None,
            _mark: PhantomData,
        }
    }

    set_attr!(set_pic,pic,ChainedPics);
    set_attr!(set_xapic,xapic,xApic);
    set_attr!(set_io_apic,io_apic,IoApic);
    set_attr!(set_local_apic,local_apic,LocalApic);
}

impl ProgrammableController<X2APIC> {
    fn new(t: X2APIC) ->Self {
        Self{
            local_apic: Some(t.0),
            io_apic: Some(t.1),
            xapic: None,
            pic: None,
            _mark: PhantomData,
        }
    }

    pub unsafe fn version(&mut self) -> String {
        format!("local apic:{}, io apic:{} ", self.local_apic.as_mut().expect("local apic not init").version(), self.io_apic.as_mut().expect("io apic not init").version())
    }

    pub unsafe fn eoi(&mut self, _number: Option<u8>) {
        self.local_apic.as_mut().expect("local apic not init").end_of_interrupt()
    }

    pub unsafe fn init(&mut self, info: ApicInfo) {
        self.local_apic = Some(LocalApic::from(info));
        self.io_apic.as_mut().expect("io apic not init").init(info.ioapic_offset.expect("missing IO APIC offset argument"))
    }

    pub unsafe fn enable(&mut self) {
        self.local_apic.as_mut().expect("local apic not init").disable()
    }

    pub unsafe fn disable(&mut self) {
        self.local_apic.as_mut().expect("local apic not init").disable()
    }

    pub unsafe fn enable_timer(&mut self) {
        self.local_apic.as_mut().expect("local apic not init").enable_timer()
    }

    pub unsafe fn disable_timer(&mut self) {
        self.local_apic.as_mut().expect("local apic not init").disable_timer()
    }

    pub unsafe fn set_timer_mode(&mut self, mode: TimerMode) {
        self.local_apic.as_mut().expect("local apic not init").set_timer_mode(mode)
    }

    pub unsafe fn set_timer_divide(&mut self, divide: TimerDivide) {
        self.local_apic.as_mut().expect("local apic not init").set_timer_divide(divide)
    }

    pub unsafe fn set_timer_initial(&mut self, initial: u32) {
        self.local_apic.as_mut().expect("local apic not init").set_timer_initial(initial)
    }

    pub unsafe fn set_logical_id(&mut self, dest: u32) {
        self.local_apic.as_mut().expect("local apic not init").set_logical_id(dest)
    }

    pub unsafe fn send_ipi(&mut self, vector: u8, dest: u32) {
        self.local_apic.as_mut().expect("local apic not init").send_ipi(vector, dest)
    }

    pub unsafe fn send_ipi_all(&mut self, vector: u8, who: IpiAllShorthand) {
        self.local_apic.as_mut().expect("local apic not init").send_ipi_all(vector, who)
    }

    pub unsafe fn send_lowest_priority_ipi(&mut self, vector: u8, dest: u32) {
        self.local_apic.as_mut().expect("local apic not init").send_lowest_priority_ipi(vector, dest)
    }

    pub unsafe fn send_lowest_priority_ipi_all(&mut self, vector: u8, who: IpiAllShorthand) {
        self.local_apic.as_mut().expect("local apic not init").send_lowest_priority_ipi_all(vector, who)
    }

    pub unsafe fn send_smi(&mut self, dest: u32) {
        self.local_apic.as_mut().expect("local apic not init").send_smi(dest)
    }

    pub unsafe fn send_smi_all(&mut self, who: IpiAllShorthand) {
        self.local_apic.as_mut().expect("local apic not init").send_smi_all(who)
    }

    pub unsafe fn send_nmi(&mut self, dest: u32) {
        self.local_apic.as_mut().expect("local apic not init").send_nmi(dest)
    }

    pub unsafe fn send_nmi_all(&mut self, who: IpiAllShorthand) {
        self.local_apic.as_mut().expect("local apic not init").send_nmi_all(who)
    }

    pub unsafe fn send_sipi(&mut self, vector: u8, dest: u32) {
        self.local_apic.as_mut().expect("local apic not init").send_ipi(vector, dest)
    }

    pub unsafe fn send_sipi_all(&mut self, vector: u8) {
        self.local_apic.as_mut().expect("local apic not init").send_sipi_all(vector)
    }

    pub unsafe fn send_ipi_self(&mut self, vector: u8) {
        self.local_apic.as_mut().expect("local apic not init").send_ipi_self(vector)
    }

    pub unsafe fn enable_irq(&mut self, irq: u8, dest: u32, mode: IrqMode, options: IrqFlags) {
        self.io_apic.as_mut().expect("io apic not init").enable_irq(irq, dest, mode, options);
    }

    pub unsafe fn disable_irq(&mut self, irq: u8) {
        self.io_apic.as_mut().expect("io apic not init").disable_irq(irq)
    }

    pub unsafe fn io_apic_set_arbitration_id(&mut self, id: u8) {
        self.io_apic.as_mut().expect("io apic not init").set_arbitration_id(id)
    }

    pub unsafe fn io_apic_set_id(&mut self, id: u8) {
        self.io_apic.as_mut().expect("io apic not init").set_id(id)
    }
}

impl ProgrammableController<PIC> {
    fn new(t: PIC) -> Self {
        Self{
            local_apic: None,
            io_apic: None,
            xapic: None,
            pic: Some(t.0),
            _mark: Default::default()
        }
    }

    pub unsafe fn version(&mut self) -> String {
        String::from("8259A")
    }

    pub unsafe fn eoi(&mut self, number: Option<u8>) {
        self.pic.as_mut().expect("pic not init").notify_end_of_interrupt(number.expect("must give notify vector"))
    }

    pub unsafe fn init(&mut self, _info: ApicInfo) {
        self.pic.as_mut().expect("pic not init").initialize()
    }

    pub unsafe fn enable(&mut self) {}

    pub unsafe fn disable(&mut self) {
        self.pic.as_mut().expect("pic not init").disable_8259a()
    }

    pub unsafe fn enable_timer(&mut self) {
        panic!("8259 not support enable_timer ")
    }

    pub unsafe fn disable_timer(&mut self) {
        panic!("8259 not support disable_timer ")
    }

    pub unsafe fn set_timer_mode(&mut self, _mode: TimerMode) {
        panic!("8259 not support set_timer_mode ")
    }

    pub unsafe fn set_timer_divide(&mut self, _divide: TimerDivide) {
        panic!("8259 not support set_timer_divide ")
    }

    pub unsafe fn set_timer_initial(&mut self, _initial: u32) {
        panic!("8259 not support set_timer_initial ")
    }

    pub unsafe fn set_logical_id(&mut self, _dest: u32) {
        panic!("8259 not support set_logical_id ")
    }

    pub unsafe fn send_ipi(&mut self, _vector: u8, _dest: u32) {
        panic!("8259 not support send_ipi ")
    }

    pub unsafe fn send_ipi_all(&mut self, _vector: u8, _who: IpiAllShorthand) {
        panic!("8259 not support send_lowest_priority_ipi ")
    }

    pub unsafe fn send_lowest_priority_ipi(&mut self, _vector: u8, _dest: u32) {
        panic!("8259 not support send_lowest_priority_ipi ")
    }

    pub unsafe fn send_lowest_priority_ipi_all(&mut self, _vector: u8, _who: IpiAllShorthand) {
        panic!("8259 not support send_lowest_priority_ipi_all ")
    }

    pub unsafe fn send_smi(&mut self, _dest: u32) {
        panic!("8259 not support send smi ")
    }

    pub unsafe fn send_smi_all(&mut self, _who: IpiAllShorthand) {
        panic!("8259 not support send smi all")
    }

    pub unsafe fn send_nmi(&mut self, _dest: u32) {
        panic!("8259 not support send nmi")
    }

    pub unsafe fn send_nmi_all(&mut self, _who: IpiAllShorthand) {
        panic!("8259 not support send nmi all")
    }

    pub unsafe fn send_sipi(&mut self, _vector: u8, _dest: u32) {
        panic!("8259 not support send sipi")
    }

    pub unsafe fn send_sipi_all(&mut self, _vector: u8) {
        panic!("8259 not support send sipi all")
    }

    pub unsafe fn send_ipi_self(&mut self, _vector: u8) {
        panic!("8259 not support send ipi self")
    }

    pub unsafe fn enable_irq(&mut self, _irq: u8, _dest: u32, _mode: IrqMode, _options: IrqFlags) {
        panic!("8259 not support irq")
    }

    pub unsafe fn disable_irq(&mut self, _irq: u8) {
        panic!("8259 not support irq")
    }

    pub unsafe fn io_apic_set_arbitration_id(&mut self, _id: u8) {
        panic!("8259 not support set arbitration id")
    }

    pub unsafe fn io_apic_set_id(&mut self, _id: u8) {
        panic!("8259 not support set set id")
    }
}

impl ProgrammableController<XPAIC> {
    fn new(t: XPAIC) -> Self {
        Self{
            local_apic: None,
            io_apic: None,
            xapic: Some(t.0),
            pic: None,
            _mark:PhantomData
        }
    }

    pub unsafe fn version(&mut self) -> String {
        format!("xapic: {}", self.xapic.as_ref().expect("xapic not init").version())
    }

    pub unsafe fn eoi(&mut self, _number: Option<u8>) {
        self.xapic.as_mut().expect("xapic not init").eoi()
    }

    pub unsafe fn init(&mut self, _info: ApicInfo) {
        self.xapic.as_mut().expect("xapic not init").cpu_init()
    }

    pub unsafe fn enable(&mut self) {
        unimplemented!()
    }
    pub unsafe fn disable(&mut self) { unimplemented!() }
    pub unsafe fn enable_timer(&mut self) {
        unimplemented!()
    }
    pub unsafe fn disable_timer(&mut self) { unimplemented!() }
    pub unsafe fn set_timer_mode(&mut self, _mode: TimerMode) {
        unimplemented!()
    }
    pub unsafe fn set_timer_divide(&mut self, _divide: TimerDivide) {
        unimplemented!()
    }
    pub unsafe fn set_timer_initial(&mut self, _initial: u32) { unimplemented!() }
    pub unsafe fn set_logical_id(&mut self, _dest: u32) {
        unimplemented!()
    }
    pub unsafe fn send_ipi(&mut self, _vector: u8, _dest: u32) {
        unimplemented!()
    }
    pub unsafe fn send_ipi_all(&mut self, _vector: u8, _who: IpiAllShorthand) {
        unimplemented!()
    }
    pub unsafe fn send_lowest_priority_ipi(&mut self, _vector: u8, _dest: u32) {
        unimplemented!()
    }
    pub unsafe fn send_lowest_priority_ipi_all(&mut self, _vector: u8, _who: IpiAllShorthand) {
        unimplemented!()
    }
    pub unsafe fn send_smi(&mut self, _dest: u32) {
        unimplemented!()
    }
    pub unsafe fn send_smi_all(&mut self, _who: IpiAllShorthand) {
        unimplemented!()
    }
    pub unsafe fn send_nmi(&mut self, _dest: u32) {
        unimplemented!()
    }
    pub unsafe fn send_nmi_all(&mut self, _who: IpiAllShorthand) {
        unimplemented!()
    }
    pub unsafe fn send_sipi(&mut self, _vector: u8, _dest: u32) {
        unimplemented!()
    }
    pub unsafe fn send_sipi_all(&mut self, _vector: u8) {
        unimplemented!()
    }
    pub unsafe fn send_ipi_self(&mut self, _vector: u8) {
        unimplemented!()
    }
    pub unsafe fn enable_irq(&mut self, _irq: u8, _dest: u32, _mode: IrqMode, _options: IrqFlags) { unimplemented!() }
    pub unsafe fn disable_irq(&mut self, _irq: u8) {
        unimplemented!()
    }
    pub unsafe fn io_apic_set_arbitration_id(&mut self, _id: u8) {
        unimplemented!()
    }
    pub unsafe fn io_apic_set_id(&mut self, _id: u8) {
        unimplemented!()
    }
}



