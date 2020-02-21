
bitflags! {
    pub struct SectionHeaderFlags: u64 {
        /// section内的数据可写
        const SHF_WRITE = 0x1;
        /// section需要占据内存
        const SHF_ALLOC = 0x2;
        /// 节内包含可以执行的机器指令
        const SHF_EXECINSTR = 0x4;
        /// 标识可以将其中包含的数据合并以消除重复的节。除非还设置了 SHF_STRINGS 标志，否则该节中的数据元素大小一致。
        /// 每个元素的大小在节头的 sh_entsize 字段中指定。如果还设置了 SHF_STRINGS 标志，则数据元素会包含以空字符结尾的字符串。每个字符的大小在节头的 sh_entsize 字段中指定
        /// 该节为只读 从单独的重定位记录可以访问该节中的每一项
        /// 如果该节还设置了 SHF_STRINGS 标志，那么该节只能包含以空字符结尾的字符串。
        /// 空字符只能作为字符串结束符，而不能出现在任何字符串的中间位置
        /// 同时设置了 SHF_STRINGS 节标志和 SHF_MERGE 标志时，
        /// 该节中的字符串就可以与其他兼容节中的字符串合并
        const SHF_MERGE = 0x10;
        /// 标识包含以空字符结尾的字符串的节。每个字符的大小在节头的 sh_entsize 字段中指定
        const SHF_STRINGS = 0x20;
        /// 此节头的 sh_info 字段中包含节头表索引
        const SHF_INFO_LINK = 0x40;
        /// 此节向链接编辑器中添加特殊排序要求
        /// 如果此节头的 sh_link 字段引用其他节（链接到的节），则会应用这些要求
        /// 如果将此节与输出文件中的其他节合并，则此节将按照相同的相对顺序（相对于这些节）显示
        /// 链接到的节也将按照相同的相对顺序（相对于与其合并的节）显示
        const SHF_LINK_ORDER = 0x80;
        /// 此节除了要求标准链接规则之外，还要求特定于操作系统的特殊处理，以避免不正确的行为
        /// 如果此节具有 sh_type 值，或者对于这些字段包含特定于操作系统范围内的 sh_flags 位，并且链接编辑器无法识别这些值，则包含此节的目标文件会由于出错而被拒绝
        const SHF_OS_NONCONFORMING = 0x100;
        /// 此节是节组的一个成员（可能是唯一的成员）。
        /// 此节必须由 SHT_GROUP 类型的节引用。
        /// 只能对可重定位目标文件中包含的节设置 SHF_GROUP 标志
        const SHF_GROUP = 0x200;
        /// 此节包含线程局部存储。进程中的每个线程都包含此数据的一个不同实例
        const SHF_TLS = 0x400;
        /// 此掩码中包括的所有位都保留用于特定于操作系统的语义
        const SHF_MASKOS = 0x0ff00000;
        /// 此节无法通过链接编辑器丢弃，并且始终会被复制到输出目标文件中
        /// 链接编辑器提供了通过链接编辑放弃未使用的输入节的功能。
        /// SHF_SUNW_NODISCARD 节标志将该节排除在此类优化之外
        const SHF_SUNW_NODISCARD = 0x00100000;
        /// 此掩码中包括的所有位都保留用于特定于处理器的语义。
        const SHF_MASKPROC = 0xf0000000;
        /// x64 的缺省编译模型仅用于 32 位位移。此位移限制了节的大小，并最终限制段的大小不得超过 2 GB
        /// 此属性标志用于标识可包含超过 2 GB 数据的节。此标志允许链接使用不同代码模型的目标文件。
        const SHF_AMD64_LARGE = 0x10000000;
        /// SHF_ORDERED 是 SHF_LINK_ORDER 所提供的旧版本功能，并且已被 SHF_LINK_ORDER 取代。
        const SHF_ORDERED = 0x40000000;
        /// 此节不包括在可执行文件或共享目标文件的链接编辑的输入中。
        /// 如果还设置了 SHF_ALLOC 标志，或者存在针对此节的重定位
        const SHF_EXCLUDE = 0x80000000;
    }
}