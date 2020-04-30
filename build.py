import os
from os import popen
from os.path import join, split, exists, isabs
from shutil import copyfile


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
        return map(lambda f: os.path.abspath(join("boot_temp", f)),
                   filter(lambda f: f.endswith(".o"), os.listdir("boot_temp")))


class Linker:

    def __init__(self, files):
        self._files = [f for f in files]
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
        assert len(self._files) >= 3
        cmd = f'{" ".join(self.flags)} {" ".join(self._files)} -o {path if path else "temp_obj"}'
        print(cmd)
        cmd = popen(cmd)
        context = cmd.read()
        print(context if context else "ld obj file succeed!")
        return path


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

    def release(self):
        self._release = True
        return self

    def build(self):
        mode = 'release' if self._release else 'debug'
        if len(self.kind) > 0:
            cmd = f"cargo xbuild --features \"{' '.join(self.kind)}\" --{'release' if self._release else ''}"
        else:
            cmd = f"cargo xbuild {'--release' if self._release else ''}"
        print(cmd)
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
        print(exists(self.kernel_file))
        if not exists(self.kernel_file):
            raise ValueError("not kernel file found")
        if not exists(self.grub_file):
            raise ValueError("not grub file found")
        # copy grub file
        copyfile(self.grub_file, join(self.grub_path, "grub.cfg"))
        # copy kernel file
        copyfile(self.kernel_file, join(join(self.base, "boot"), split(self.kernel_file)[1]))
        popen(f"grub-mkrescue -o {iso_file} {self.base}")


if __name__ == '__main__':
    # fs = Nasm().elf64().dirs("src/boot").run()
    fs = Nasm().use_temp_file()
    p = KernelBuilder() \
        .build()
    kernel_file = Linker(fs) \
        .no_magic() \
        .script_file("linker.ld") \
        .add_file(p) \
        .link("kernel.bin")
    Grub() \
        .add_kernel_file(kernel_file) \
        .add_grub_path("src/boot/grub.cfg") \
        .mkiso_file("os.iso")
