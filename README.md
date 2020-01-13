# Operating-System
Operating System Notes

本项目是使用Rust编写的微内核x64位操作系统，支持多进程调度，鼠标操作，键盘输入等（其余功能还在开发中）

# 博客地址

[使用Rust开发操作系统](https://blog.csdn.net/qq_41698827)


# 编写环境
## 系统
Linux version 5.0.0-27-generic (buildd@lgw01-amd64-031) (gcc version 7.4.0 (Ubuntu 7.4.0-1ubuntu1~18.04.1)) #28~18.04.1-Ubuntu

## 虚拟机
### qemu-system-x86

```
sudo apt install qemu-system-x86
```

## RUST 编译器
rustc 1.41.0-nightly (ded5ee001 2019-11-13)

### 安装

#### rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### nightly安装（需要安装rustup 一般安装完rust后自带的）
```
rustup install nightly
```

#### 查看
```
$>> rustup toolchain list
stable-x86_64-pc-windows-msvc (default)
beta-x86_64-pc-windows-msvc
nightly-x86_64-pc-windows-msvc
```

#### 切换编译器
```
// 切换到cargo项目目录
$ rustup override set nightly
```

### 安装xbuild

```
cargo install cargo-xbuild
```

# UEFI加载
## 固件制作
1. 准备[EDK2](https://github.com/tianocore/edk2) 
```
$ git clone https://github.com/tianocore/edk2.git
$ cd edk2
// EDK2有一些依赖库比如openssl等
$ git submodule update --init
```
2. 指定开发环境
clone完毕后在`edk2/Conf/target.txt`更改所需平台

例如Ubuntu(64) 
```
ACTIVE_PLATFORM       =  EmulatorPkg/EmulatorPkg.dsc
                                             DEBUG platform target.
TARGET                = DEBUG

TARGET_ARCH           = IA32

TOOL_CHAIN_CONF       = Conf/tools_def.txt
                                           used for the build.  The list uses space character separation.
TOOL_CHAIN_TAG        = GCC5 
                                                the default value in this file.
# MAX_CONCURRENT_THREAD_NUMBER = 1

BUILD_RULE_CONF = Conf/build_rule.txt
```

3. 编译EDK2工具链
安装依赖环境
```
sudo apt-get install build-essential uuid-dev
```
开始编译
```
edk2$ cd BaseTools
edk2/BaseTools$ make
```
4. 启用工具

```
edk2$ source edksetup.sh
```

5. 编译OVMF
编译64位固件
```
edk2$ build -a X64 -p OvmfPkg/OvmfPkgX64.dsc -t GCC5
```
编译完毕后将`OVMF.fd`, `OVMF_CODE.fd`, `OVMF_VARS.fd`拷贝至`src/uefis/`

6. 编译内核
```
$ cd src/kernel
$ make build
```
将`/target/x86-64/debug/bootimage-kernel.bin` 重命名为 kernel.bin 拷贝到
`src/uefis/target/x86_64-unknown-uefi/debug/esp/EFI/Boot/`中

## 启动QEUM
在`src/uefi`文件中执行以下命令
```
python3 build.py run 
```


# 参考书籍

《一个64位操作系统的设计与实现》

《30天自制操作系统》

《Orange\`s一个操作系统的实现》

《Linux内核完全注释》

《Professional Assembly Language》

《汇编语言（第三版）》

《深入理解Linux内核》

《深入理解BootLoader》

《深入理解计算机系统第三版》

《操作系统概念》

《64-ia-32-architectures-software-developer-vol-3a》

# 网站

http://www.maverick-os.dk/

http://mikeos.sourceforge.net/write-your-own-os.html

http://www.bioscentral.com

# License
Source code  is under the Apache License.
The `src/lib` code is reference [rust-osdev](https://github.com/rust-osdev/x86_64)

# Todo

- [x] println!和print！宏
- [x] 原子锁
- [x] 异常处理
- [x] 双重异常处理
- [x] 初级中断处理（8259A）
- [x] 键盘可输入ASCII可打印字符
- [x] UEFI启动
- [x] 处理器信息处理
- [x] 分页内存
- [ ] 初级进程
- [ ] 添加系统调用
- [ ] 进程间通信(IPC)
- [ ] 鼠标驱动
- [ ] SLAB内存池以及通用内存管理
- [ ] 文件系统
- [ ] Shell解析器
- [ ] 系统API库