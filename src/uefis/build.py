#!/usr/bin/env python3

'Script used to build, run, and test the code on all supported platforms.'

import argparse
import filecmp
import json
import os
import re
import shutil
import subprocess as sp
import sys
from pathlib import Path

# 工作目录，以build.py文件为准，应该与Cargo.toml文件同级
WORKSPACE_DIR = Path(__file__).resolve().parents[0]

# 基本的设置
SETTINGS = {
    # 在运行之前显示执行的命令
    'verbose': True,
    # QEMU不以GUI模式运行
    'headless': False,
    # 编译目标平！！！不可修改！！！
    'target': 'x86_64-unknown-uefi',
    # 构建标识 debug 和 release
    'config': 'debug',
    # QEMU 启动命令
    'qemu_binary': 'qemu-system-x86_64',
    # OVMF固件地址 可指定
    'ovmf_dir': WORKSPACE_DIR,
    # 编译生成的名称,修改该名称必须同时修改整个目录名称
    "uefi_name": 'uefis.efi',
    # 项目名称（目录名称）
    'project_name': 'uefis'
}


def build_dir():
    return WORKSPACE_DIR / 'target' / SETTINGS['target'] / SETTINGS['config']


def esp_dir():
    return build_dir() / 'esp'


def run_xtool(tool, *flags):
    cmd = ['cargo', tool, '--target', SETTINGS['target'], *flags]

    if SETTINGS['verbose']:
        print(' '.join(cmd))

    sp.run(cmd).check_returncode()


def run_xbuild(*flags):
    run_xtool('xbuild', *flags)


def run_xclippy(*flags):
    run_xtool('xclippy', *flags)


def build(*test_flags):
    xbuild_args = [
        '--package', SETTINGS['project_name'],
        *test_flags,
    ]

    if SETTINGS['config'] == 'release':
        xbuild_args.append('--release')

    run_xbuild(*xbuild_args)

    # Copy the built test runner file to the right directory for running tests.
    built_file = build_dir() / SETTINGS['uefi_name']

    boot_dir = esp_dir() / 'EFI' / 'Boot'

    boot_dir.mkdir(parents=True, exist_ok=True)

    output_file = boot_dir / 'BootX64.efi'

    shutil.copy2(built_file, output_file)


def clippy():
    run_xclippy('--all')


def doc():
    sp.run([
        'cargo', 'doc', '--no-deps',
        '--package', 'uefi',
        '--package', 'uefi-exts',
        '--package', 'uefi-alloc',
        '--package', 'uefi-logger',
        '--package', 'uefi-services',
    ])


def run_qemu():
    build()

    ovmf_dir = SETTINGS['ovmf_dir']
    ovmf_code, ovmf_vars = ovmf_dir / 'OVMF_CODE.fd', ovmf_dir / 'OVMF_VARS.fd'

    if not ovmf_code.is_file():
        raise FileNotFoundError(f'OVMF_CODE.fd not found in the `{ovmf_dir}` directory')

    examples_dir = build_dir() / 'examples'

    qemu_monitor_pipe = 'qemu-monitor'

    qemu_flags = [
        # Disable default devices.
        # QEMU by defaults enables a ton of devices which slow down boot.
        '-nodefaults',

        # Use a modern machine, with acceleration if possible.
        '-machine', 'q35,accel=kvm:tcg',

        # Multi-processor services protocol test needs exactly 3 CPUs.
        '-smp', '3',

        # Allocate some memory.
        '-m', '128M',

        # Set up OVMF.
        '-drive', f'if=pflash,format=raw,file={ovmf_code},readonly=on',
        '-drive', f'if=pflash,format=raw,file={ovmf_vars},readonly=on',

        # Mount a local directory as a FAT partition.
        '-drive', f'format=raw,file=fat:rw:{esp_dir()}',

        # Mount the built examples directory.
        '-drive', f'format=raw,file=fat:rw:{examples_dir}',

        # Connect the serial port to the host. OVMF is kind enough to connect
        # the UEFI stdout and stdin to that port too.
        '-serial', 'stdio',

        # Map the QEMU exit signal to port f4
        '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',

        # Map the QEMU monitor to a pair of named pipes
        '-qmp', f'pipe:{qemu_monitor_pipe}',

        # OVMF debug builds can output information to a serial `debugcon`.
        # Only enable when debugging UEFI boot:
        # '-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ]

    # When running in headless mode we don't have video, but we can still have
    # QEMU emulate a display and take screenshots from it.
    qemu_flags.extend(['-vga', 'std'])
    if SETTINGS['headless']:
        # Do not attach a window to QEMU's display
        qemu_flags.extend(['-display', 'none'])

    cmd = [SETTINGS['qemu_binary']] + qemu_flags

    if SETTINGS['verbose']:
        print(' '.join(cmd))

    # This regex can be used to detect and strip ANSI escape codes when
    # analyzing the output of the test runner.
    ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]')

    # Setup named pipes as a communication channel with QEMU's monitor
    monitor_input_path = f'{qemu_monitor_pipe}.in'
    if not os.path.exists(monitor_input_path):
        os.mkfifo(monitor_input_path)
    if not os.path.exists(qemu_monitor_pipe):
        monitor_output_path = f'{qemu_monitor_pipe}.out'
    if not os.path.exists(monitor_output_path):
        os.mkfifo(monitor_output_path)

    # Start QEMU
    qemu = sp.Popen(cmd, stdin=sp.PIPE, stdout=sp.PIPE, universal_newlines=True)
    try:
        # Connect to the QEMU monitor
        with open(monitor_input_path, mode='w') as monitor_input, \
                open(monitor_output_path, mode='r') as monitor_output:
            # Execute the QEMU monitor handshake, doing basic sanity checks
            assert monitor_output.readline().startswith('{"QMP":')
            print('{"execute": "qmp_capabilities"}', file=monitor_input, flush=True)
            assert monitor_output.readline() == '{"return": {}}\n'

            # Iterate over stdout...
            for line in qemu.stdout:
                # Strip ending and trailing whitespace + ANSI escape codes
                # (This simplifies log analysis and keeps the terminal clean)
                stripped = ansi_escape.sub('', line.strip())

                # Skip lines which contain nothing else
                if not stripped:
                    continue

                # Print out the processed QEMU output for logging & inspection
                print(stripped)

                # If the app requests a screenshot, take it
                if stripped.startswith("SCREENSHOT: "):
                    reference_name = stripped[12:]

                    # Ask QEMU to take a screenshot
                    monitor_command = '{"execute": "screendump", "arguments": {"filename": "screenshot.ppm"}}'
                    print(monitor_command, file=monitor_input, flush=True)

                    # Wait for QEMU's acknowledgement, ignoring events
                    reply = json.loads(monitor_output.readline())
                    while "event" in reply:
                        reply = json.loads(monitor_output.readline())
                    assert reply == {"return": {}}

                    # Tell the VM that the screenshot was taken
                    print('OK', file=qemu.stdin, flush=True)

                    # Compare screenshot to the reference file specified by the user
                    # TODO: Add an operating mode where the reference is created if it doesn't exist
                    reference_file = WORKSPACE_DIR / 'uefi-test-runner' / 'screenshots' / (reference_name + '.ppm')
                    assert filecmp.cmp('screenshot.ppm', reference_file)

                    # Delete the screenshot once done
                    os.remove('screenshot.ppm')
    finally:
        try:
            # Wait for QEMU to finish
            status = qemu.wait(15)
        except sp.TimeoutExpired:
            print('Tests are taking too long to run, killing QEMU', file=sys.stderr)
            qemu.kill()
            status = -1

        # Delete the monitor pipes
        os.remove(monitor_input_path)
        os.remove(monitor_output_path)

        # Throw an exception if QEMU failed
        if status != 0:
            raise sp.CalledProcessError(cmd=cmd, returncode=status)


def main():
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    usage = '%(prog)s verb [options]'
    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(usage=usage, description=desc)

    parser.add_argument('verb', help='command to run', type=str,
                        choices=['build', 'run', 'doc', 'clippy'])

    parser.add_argument('--verbose', '-v', help='print commands before executing them',
                        action='store_true')

    parser.add_argument('--headless', help='run QEMU without a GUI',
                        action='store_true')

    parser.add_argument('--release', help='build in release mode',
                        action='store_true')

    opts = parser.parse_args()

    # Check if we need to enable verbose mode
    SETTINGS['verbose'] = opts.verbose
    SETTINGS['headless'] = opts.headless
    SETTINGS['config'] = 'release' if opts.release else 'debug'

    verb = opts.verb

    if verb == 'build':
        build()
    elif verb == 'clippy':
        clippy()
    elif verb == 'doc':
        doc()
    elif verb == 'run' or verb is None or opts.verb == '':
        # Run the program, by default.
        run_qemu()
    else:
        raise ValueError(f'Unknown verb {opts.verb}')


if __name__ == '__main__':
    try:
        main()
    except sp.CalledProcessError as cpe:
        print(f'Subprocess {cpe.cmd[0]} exited with error code {cpe.returncode}')
        sys.exit(1)
