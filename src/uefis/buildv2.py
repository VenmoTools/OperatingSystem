import argparse
import filecmp
import json
import os
import re
import shutil
import subprocess as sp
import sys
from pathlib import Path

# 获取当前文件执行路径，一般放在Cargo.toml同级目录
WORKSPACE = Path(__file__).resolve().parents[0]

GlobalSetting = {
    # Print commands before running them.
    'verbose': False,
    # Run QEMU without showing GUI
    'headless': False,
    # Target to build for.
    'target': 'x86_64-unknown-uefi',
    # Configuration to build.
    'config': 'debug',
    # QEMU executable to use
    'qemu_binary': 'qemu-system-x86_64',
    # Path to directory containing `OVMF_{CODE/VARS}.fd`.
    # TODO: use installed OVMF, if available.
    'ovmf_dir': WORKSPACE,
    "uefi_name": 'uefis.efi',
    'project_name': 'uefis',
    'output_filename': "Bootx64.efi",
}

ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]')


class Build:
    build_path = WORKSPACE / "target" / GlobalSetting["target"] / GlobalSetting["config"]
    esp_path = build_path / "esp"
    built_file = build_path / GlobalSetting["uefi_name"]
    boot_dir = esp_path / "EIF" / "Boot"
    output_file = boot_dir / GlobalSetting["output_filename"]
    examples_dir = build_path / 'examples'

    def __init__(self):
        ovmf_code, ovmf_vars = self.find_ovmf()
        self.qemu_monitor_pipe = 'qemu-monitor'
        self.qemu_flags = [
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
            '-drive', f'format=raw,file=fat:rw:{self.esp_path}',

            # Mount the built examples directory.
            '-drive', f'format=raw,file=fat:rw:{self.examples_dir}',

            # Connect the serial port to the host. OVMF is kind enough to connect
            # the UEFI stdout and stdin to that port too.
            '-serial', 'stdio',

            # Map the QEMU exit signal to port f4
            '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',

            # Map the QEMU monitor to a pair of named pipes
            '-qmp', f'pipe:{self.qemu_monitor_pipe}',

            # OVMF debug builds can output information to a serial `debugcon`.
            # Only enable when debugging UEFI boot:
            # '-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
        ]

    @staticmethod
    def run_script(tool, *flags):
        cmd = ["cargo", tool, "--target", *flags]
        if GlobalSetting["verbose"]:
            print(" ".join(cmd))
        sp.run(cmd).check_returncode()

    def build_script(self, *flags):
        self.run_script("xbuild", *flags)

    def rebuild_path(self):
        self.boot_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(self.built_file, self.output_file)

    def build(self, *flags):
        build_args = [
            "--package", GlobalSetting['project_name'],
            *flags,
        ]
        if GlobalSetting["config"] == "release":
            build_args.append("--release")
        self.build_script(*build_args)

    def find_ovmf(self):
        ovmf_dir = GlobalSetting['ovmf_dir']
        ovmf_code, ovmf_vars = ovmf_dir / 'OVMF_CODE.fd', ovmf_dir / 'OVMF_VARS.fd'
        if not ovmf_code.is_file():
            raise FileNotFoundError(f'OVMF_CODE.fd not found in the `{ovmf_dir}` directory')
        return ovmf_code, ovmf_vars

    def create_pipline(self):
        monitor_input_path = f'{self.qemu_monitor_pipe}.in'
        if not os.path.exists(monitor_input_path):
            os.mkfifo(monitor_input_path)
        if not os.path.exists(self.qemu_monitor_pipe):
            monitor_output_path = f'{self.qemu_monitor_pipe}.out'
        if not os.path.exists(monitor_output_path):
            os.mkfifo(monitor_output_path)

        return monitor_input_path, monitor_output_path

    def handle_screenshot(self, qemu, output, _input, stripped):
        reference_name = stripped[12:]
        monitor_command = '{"execute": "screendump", "arguments": {"filename": "screenshot.ppm"}}'
        print(monitor_command, file=_input, flush=True)
        reply = json.loads(output.readline())
        while "event" in reply:
            reply = json.loads(output.readline())
        assert reply == {"return": {}}
        print('OK', file=qemu.stdin, flush=True)
        reference_file = WORKSPACE / 'uefi-test-runner' / 'screenshots' / (reference_name + '.ppm')
        assert filecmp.cmp('screenshot.ppm', reference_file)
        os.remove('screenshot.ppm')

    def start_qemu(self):
        self.build()

        self.qemu_flags.extend(['-vga', 'std'])
        if GlobalSetting["headless"]:
            self.qemu_flags.extend(['-display', 'none'])

        cmd = [GlobalSetting["qemu_binary"]] + self.qemu_flags

        if GlobalSetting['verbose']:
            print(' '.join(cmd))

        # 创建Piping
        monitor_input_path, monitor_output_path = self.create_pipline()
        # 启动QEMU
        qemu = sp.Popen(cmd, stdin=sp.PIPE, stdout=sp.PIPE, universal_newlines=True)
        try:
            with open(monitor_input_path, "w") as m_input, \
                    open(monitor_output_path) as m_output:
                assert m_output.readline().startswith('{"QMP":')
                print('{"execute": "qmp_capabilities"}', file=m_input, flush=True)
                assert m_output.readline() == '{"return": {}}\n'

                for line in qemu.stdout:
                    stripped = ansi_escape.sub('', line.strip())
                    if not stripped:
                        continue
                    print(stripped)
                    if stripped.startswith("SCREENSHOT: "):
                        self.handle_screenshot(qemu, m_output, m_input, stripped)
        finally:
            try:
                status = qemu.wait(15)
            except sp.TimeoutExpired:
                print('Tests are taking too long to run, killing QEMU', file=sys.stderr)
                qemu.kill()
                status = -1
            os.remove(monitor_input_path)
            os.remove(monitor_output_path)

            if status != 0:
                raise sp.CalledProcessError(cmd=cmd, returncode=status)

    def parser(self):
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
        GlobalSetting['verbose'] = opts.verbose
        GlobalSetting['headless'] = opts.headless
        GlobalSetting['config'] = 'release' if opts.release else 'debug'

        verb = opts.verb

        if verb == 'build':
            self.build()
        elif verb == 'run' or verb is None or opts.verb == '':
            # Run the program, by default.
            self.start_qemu()
        else:
            raise ValueError(f'Unknown verb {opts.verb}')


if __name__ == '__main__':
    try:
        Build().parser()
    except sp.CalledProcessError as cpe:
        print(f'Subprocess {cpe.cmd[0]} exited with error code {cpe.returncode}')
        sys.exit(1)
