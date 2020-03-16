pub mod lvt;

pub struct LocalAPIC {
    /// 版本ID
    vendor: u8,
    /// 保留位
    reserved_1: u8,
    /// LVT表项数目 此数值+1代表处理器支持的LVT表项数
    lvt_entry: u8,
    /// reserved_2中包含禁止广播EOI消息标志位因此取值只能为1或0
    reserved_2: u8,
}

#[derive(Debug, Copy, Clone)]
pub enum LocalAPICVendor {
    _82489DX,
    IntegratedAPIC,
}

