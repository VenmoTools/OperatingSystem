import os
import shutil
from os import popen
from os.path import join, split, exists, isabs
from shutil import copyfile
from sys import stderr

import toml

WORK_PATH = split(os.path.abspath(__file__))[0]
KERNEL_PATH = join(WORK_PATH, "kernel")
UEFI_PATH = join(WORK_PATH, "uefis")


class Nasm:

    def __init__(self, flags="debug"):
        self.args = ["nasm"]
        self._files = []
        self.temp_dir = os.path.join(os.path.abspath("target"), f"{flags}/asm")
        os.makedirs(self.temp_dir, exist_ok=True)

    def add_arg(self, arg: str):
        self.args.append(arg)
        return self

    def temp(self, f):
        file = os.path.split(f)[1]

        return f'-o {os.path.join(self.temp_dir, file.replace(".asm", ".o"))}'

    def run(self):
        for f in self._files:
            cmd = f"{' '.join(self.args)} {f} {self.temp(f)}"
            print(cmd)
            cmd = popen(cmd)
            content = cmd.read()
            print(content) if content else print(f"compile {os.path.split(f)[1]} ...done!")
        return sorted(map(lambda f: self.temp(f).replace("-o ", ""), self._files), reverse=True)

    def dirs(self, dir_name):
        all_files = os.listdir(dir_name)
        res = map(lambda f: os.path.join(os.path.abspath(dir_name), f),
                  filter(lambda f: f.endswith(".asm"), all_files))
        self.files(res)
        return self

    def file(self, f: str):
        self._files.append(f)
        return self

    def files(self, files):
        for f in files:
            self._files.append(f)

    def elf64(self):
        self.args.append("-f elf64")
        return self

    @staticmethod
    def use_temp_file():
        path = os.path.abspath(
            join(
                KERNEL_PATH,
                next(filter(
                    lambda f: f == "boot_temp", os.listdir(KERNEL_PATH)
                ))
            )
        )
        return map(lambda f: os.path.abspath(join(path, f)),
                   filter(lambda f: f.endswith(".o"), os.listdir(path)))


class Linker:

    def __init__(self, files=None):
        self._files = [f for f in files] if files else []
        self.flags = ["ld"]

    def script_file(self, file):
        self.flags.append(f"-T {os.path.abspath(file)}")
        return self

    def add_file(self, f):
        self._files.append(f)
        return self

    def no_magic(self):
        self.flags.append("--nmagic")
        return self

    def link(self, path):
        if self._files:
            cmd = f'{" ".join(self.flags)} {" ".join(self._files)} -o {path if path else "temp_obj"}'
        else:
            cmd = f'{" ".join(self.flags)}  -o {path}'
        print(cmd)
        cmd = popen(cmd)
        context = cmd.read()
        print(context if context else "ld obj file succeed!")
        return path


class UefiBuilder:

    def __init__(self):
        self._release = False

    def release(self):
        self._release = True
        return self

    def build(self):
        mode = 'release' if self._release else 'debug'
        cmd = f'cargo xbuild  --target "x86_64-unknown-uefi" --{"release" if self._release else ""}'
        os.chdir(UEFI_PATH)
        cmd = popen(cmd)
        context = cmd.read()
        print(context if context else "build efi... done!")
        return os.path.join(os.path.abspath("target"), f"x86_64-unknown-uefi/{mode}/droll_os.efi")


class KernelBuilder:

    def __init__(self):
        self.kind = []
        self._release = False

    def xapic(self):
        self.kind.append("xapic")
        return self

    def x2apic(self):
        self.kind.append("x2apic")
        return self

    def pic(self):
        self.kind.append("pic")
        return self

    def uefi_kernel(self):
        self.kind.append("uefi")
        return self

    def mutiboot_kernel(self):
        self.kind.append("mutiboot")
        return self

    def with_features(self, f: list):
        self.kind.extend(f)
        return self

    def release(self):
        self._release = True
        return self

    def build(self):
        mode = 'release' if self._release else 'debug'
        if len(self.kind) > 0:
            cmd = f"cargo xbuild --features \"{' '.join(self.kind)}\" --{'release' if self._release else ''}"
        else:
            cmd = f"cargo xbuild {'--release' if self._release else ''}"
        os.chdir(KERNEL_PATH)
        cmd = popen(cmd)
        context = cmd.read()
        print(context if context else "build kernel... done!")
        return os.path.join(os.path.abspath("target"), f"x86_64-unknown-none/{mode}/librslib.a")


class Grub:

    def __init__(self):
        self.kernel_file = ""
        self.grub_file = ""
        self.base = join(os.path.abspath("target"), "iosfiles")
        self.grub_path = join(self.base, "boot/grub")
        os.makedirs(self.grub_path, exist_ok=True)

    def add_kernel_file(self, f):
        self.kernel_file = f if isabs(f) else os.path.abspath(f)
        return self

    def add_grub_path(self, f):
        self.grub_file = f
        return self

    def mkiso_file(self, iso_file):
        if not exists(self.kernel_file):
            raise ValueError("not kernel file found")
        if not exists(self.grub_file):
            raise ValueError("not grub file found")
        # copy grub file
        copyfile(self.grub_file, join(self.grub_path, "grub.cfg"))
        # copy kernel file
        copyfile(self.kernel_file, join(join(self.base, "boot"), split(self.kernel_file)[1]))
        popen(f"grub-mkrescue -o {iso_file} {self.base}")


class Unreachable(Exception):
    pass


class AreaMixin:

    def field(self, name):
        if hasattr(self, "args") and hasattr(self, "name"):
            try:
                return self.args[name]
            except KeyError:
                show_err(f"could't find {name} in {self.name} area")
                exit(1)
        raise Unreachable(f"{self.__class__}")

    def output(self):
        output = self.field("output")
        output = output if '.' not in output else output.replace('.', '')
        return join(WORK_PATH, output)


class Area(AreaMixin):

    def __init__(self, name, args):
        self.name = name
        self.args = args

    def is_used(self) -> bool:
        return self.field("use")

    def link_file(self):
        pass

    def process(self, **kwargs):
        pass


class UefiArea(Area):

    def __init__(self, args):
        super().__init__("uefi", args)

    def link_file(self) -> str:
        return os.path.abspath("linker_file/uefi.ld")

    def process(self, **kwargs):
        p = KernelBuilder() \
            .uefi_kernel() \
            .with_features(self.field("features"))
        if self.field("release"):
            p.release()
        kernel_file = Linker() \
            .script_file(self.link_file()) \
            .add_file(p.build()) \
            .link(join(WORK_PATH, "kernel_file"))

        efi = UefiBuilder()
        if self.field("release"):
            efi.release()
        efi.build()
        copyfile(join(UEFI_PATH,
                      f"target/x86_64-unknown-uefi/{'release' if self.field('release') else 'debug'}/droll_os.efi"),
                 join(self.output(), "droll_os.efi"))
        try:
            copyfile(kernel_file, join(self.output(), "kernel_file"))
        except shutil.SameFileError:
            pass


class MultiBootArea(Area):

    def __init__(self, args):
        super().__init__("multiboot", args)

    def use_tempfile(self) -> bool:
        return self.field("tempfile")

    def link_file(self) -> str:
        return os.path.abspath("linker_file/multi.ld")

    def process(self, **kwargs):
        kernel, iso = kwargs["kernel"], kwargs["iso"]
        fs = Nasm() \
            .elf64() \
            .dirs(join(WORK_PATH, "kernel/src/boot")).run() if not self.use_tempfile() else Nasm().use_temp_file()
        p = KernelBuilder() \
            .with_features(self.field("features"))
        if self.field("release"):
            p.release()
        kernel_file = Linker(fs) \
            .no_magic() \
            .script_file(self.link_file()) \
            .add_file(p.build()) \
            .link(join(WORK_PATH, "kernel.bin"))
        Grub() \
            .add_kernel_file(kernel_file) \
            .add_grub_path(join(WORK_PATH, "kernel/src/boot/grub.cfg")) \
            .mkiso_file(join(kernel.output(),
                             f"{iso.iso_name if iso.iso_name.endswith('.iso') else iso.iso_name.replace('.iso', '')}.iso")
                        )


class IsoArea(AreaMixin):

    def __init__(self, args):
        self.args = args
        self.name = "iso"

    @property
    def iso_name(self):
        return self.field("name")


class KernelArea(AreaMixin):

    def __init__(self, args):
        self.args = args
        self.name = "kernel"


class Executor(AreaMixin):

    def __init__(self, filename):
        self.name = "build.toml"
        self.args = toml.load(filename)

    def check_file(self) -> (MultiBootArea, UefiArea):
        try:
            multi = MultiBootArea(self.field("multiboot"))
            efi = UefiArea(self.field("uefi"))
            self._check_used(multi, efi)
            return multi, efi
        except KeyError:
            show_err("could find [multi] or [uefi] area in build.toml")

    @staticmethod
    def _check_used(multi: MultiBootArea, efi: UefiArea):
        assert multi.is_used() != efi.is_used(), "could't use/unused uefi and multiboot in same time"

    def process(self):
        iso = IsoArea(self.field("iso"))
        kernel = KernelArea(self.field("kernel"))
        multi, efi = self.check_file()
        self.call(multi.process, efi.process, multi.is_used(), (iso, kernel))

    @staticmethod
    def call(func1, func2, condition: bool, args: tuple):
        args1, args2 = args
        if condition:
            return func1(iso=args1, kernel=args2)
        func2(iso=args1, kernel=args2)


def show_err(*args):
    print(*args, file=stderr)


if __name__ == '__main__':
    builder = Executor("build.toml")
    builder.process()
