use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, };
use std::process::{Command, exit, ExitStatus, Stdio};

fn main() -> std::io::Result<()> {
    let work_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // 1. build kernel
    check_status(kernel_build_step(work_dir)?);
    // 2. build efi file
    check_status(efi_build_step(work_dir)?);
    // 3. copy file
    copy_file(work_dir)?;
    run_qemu(work_dir);
    Ok(())
}

fn check_status(status: ExitStatus) {
    if !status.success() {
        println!("status is not succeed: {}", status);
        exit(1);
    }
}

pub fn kernel_build_step(path: &Path) -> std::io::Result<ExitStatus> {
    Command::new("cargo")
        .current_dir(path.join("kernel"))
        .arg("xbuild")
        .arg("--release").status()
}

pub fn efi_build_step(path: &Path) -> std::io::Result<ExitStatus> {
    Command::new("cargo")
        .current_dir(path.join("uefis"))
        .arg("xbuild")
        .args(&["--package", "uefis"])
        .arg("--release")
        .status()
}

pub fn copy_file(path: &Path) -> std::io::Result<()> {
    //构建内核路径 $WORK_DIR/kernel/target/x86-64/debug/kernel
    let src_kernel_path = path.join(r"kernel\target\x86_64-unknown-none\release\kernel");
    //构建efi文件路径 $WORK_DIR/uefis/target/x86_64-unknown-uefi/debug/uefis.efi
    let uefi_path = path.join(r"uefis\target\x86_64-unknown-uefi\release\uefis.efi");
    // 构建uefi启动目录 $WORK_DIR/target/debug/esp/EFI/Boot
    let dest_path = path.join(r"target\debug\esp\EFI\Boot");

    // 创建esp/EFI/Boot目录
    std::fs::create_dir_all(&dest_path)?;

    // 复制efi文件
    let efi_file_path = dest_path.join("BootX64.efi");
    File::create(&efi_file_path)?;
    std::fs::copy(uefi_path, efi_file_path)?;

    // 复制内核文件
    let dest_kernel_path = dest_path.join("kernel");
    File::create(&dest_kernel_path)?;
    std::fs::copy(src_kernel_path, dest_kernel_path)?;

    Ok(())
}

pub fn run_qemu(path: &Path) {
    let p_code = format!("if=pflash,format=raw,file={},readonly=on", path.join("OVMF_CODE.fd").to_str().unwrap());
    let p_vars = format!("if=pflash,format=raw,file={},readonly=on", path.join("OVMF_VARS.fd").to_str().unwrap());
    let p_esp = format!("format=raw,file=fat:rw:{}", path.join("target\\debug\\esp").to_str().unwrap());
    let process = Command::new("qemu-system-x86_64.exe").stdout(Stdio::piped())
        .args(&[
            "-drive", p_code.as_str(),
            "-drive", p_vars.as_str(),
            "-drive", p_esp.as_str(),
            "-serial", "stdio",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-debugcon", "file:debug.log",
            "-s",
            "-d","cpu_reset",
            "-D","kernel/qemu.log"
//            "-S"
//            "-global", "isa-debugcon.iobase=0x402"
        ])
        .spawn().unwrap();

    let mut line = String::new();
    if let Some(out) = process.stdout {
        let mut reader = BufReader::new(out);
        while let Ok(size) = reader.read_line(&mut line) {
            if size == 0 {
                break;
            }
            println!("{}", line);
        }
    }
}