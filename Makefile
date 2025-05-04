SHELL := /bin/bash
BIN   := riscv64-qemu
NVME := off
NET  := off
LOG  := error
RELEASE := release
SMP := 1
QEMU_EXEC ?= 
GDB  ?= gdb-multiarch
ARCH := aarch64
TARGET := aarch64-unknown-none-softfloat

BUS  := device
ifeq ($(ARCH), x86_64)
  QEMU_EXEC += qemu-system-x86_64 \
				-machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2
  BUS := pci
else ifeq ($(ARCH), riscv64)
  QEMU_EXEC += qemu-system-$(ARCH) \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  QEMU_EXEC += qemu-system-$(ARCH) \
				-cpu cortex-a72 \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
  QEMU_EXEC += qemu-system-$(ARCH) -kernel $(KERNEL_ELF)
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif

KERNEL_ELF = target/$(TARGET)/$(RELEASE)/rsel4
KERNEL_BIN = $(KERNEL_ELF).bin
FS_IMG  := mount.img
features:= 
QEMU_EXEC += -m 1G\
			-nographic \
			-smp $(SMP) \
			-D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors

TESTCASE := testcase-$(ARCH)

ifeq ($(BLK), on)
QEMU_EXEC += -drive file=$(FS_IMG),if=none,format=raw,id=x0
	QEMU_EXEC += -device virtio-blk-$(BUS),drive=x0
endif

ifeq ($(NET), on)
QEMU_EXEC += -netdev user,id=net0,hostfwd=tcp::6379-:6379,hostfwd=tcp::2222-:2222,hostfwd=tcp::2000-:2000,hostfwd=tcp::8487-:8487,hostfwd=tcp::5188-:5188,hostfwd=tcp::12000-:12000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
	-device virtio-net-$(BUS),netdev=net0
features += net
endif

all: build

fs-img:
	rm -f $(FS_IMG)
	dd if=/dev/zero of=$(FS_IMG) bs=1M count=40
	mkfs.vfat -F 32 $(FS_IMG)
	sync
	sudo mount $(FS_IMG) mount -o uid=1000,gid=1000
	touch mount/file123
	mkdir mount/dir123
	sudo umount mount

env:
	rustup component add llvm-tools-preview

.PHONY: build example
build:
	cargo build --release --target $(TARGET) -p rsel4
	rust-objcopy --binary-architecture=$(ARCH) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

example:
	cargo build --release --target $(TARGET) -p example
	rust-objcopy --binary-architecture=$(ARCH) target/$(TARGET)/$(RELEASE)/example --strip-all -O binary target/$(TARGET)/$(RELEASE)/example.bin

fdt:
	$(QEMU_EXEC) -machine virt,dumpdtb=virt.out
	fdtdump virt.out

debug: build
	@tmux new-session -d \
	"$(QEMU_EXEC) -s -S && echo '按任意键继续' && read -n 1" && \
	tmux split-window -h "$(GDB) $(KERNEL_ELF) -ex 'target remote localhost:1234' -ex 'disp /16i $pc' " && \
	tmux -2 attach-session -d

clean:
	rm -rf target/

fmt:
	cargo fmt
	cd users && cargo fmt

# 在安装 sel4-kernel-loader 的时候需要指定 CC
# 否则会使用默认的 GCC，如果 host 是 x86_64 那么就会无法编译出对应架构的代码
# 参考链接： https://docs.rs/cc/latest/cc/
test: build example
	cp $(KERNEL_ELF) build/bin/kernel.elf
	CC=aarch654-linux-gnu-gcc SEL4_PREFIX=$(realpath build) cargo install \
		-Z build-std=core,compiler_builtins \
		-Z build-std-features=compiler-builtins-mem \
		--target aarch64-unknown-none \
		--git https://github.com/reL4team2/rust-sel4.git \
		--rev 642b58d807c5e5fc22f0c15d1467d6bec328faa9 \
		--root build/ \
		sel4-kernel-loader
	sel4-kernel-loader-add-payload \
		--loader build/bin/sel4-kernel-loader \
		--sel4-prefix build/ \
		--app target/$(TARGET)/$(RELEASE)/example \
		-o kernel.elf
	qemu-system-aarch64 \
		-machine virt \
		-machine virtualization=on\
		-cpu cortex-a72 \
		-kernel kernel.elf \
		-m 1G \
		-nographic \
		-serial mon:stdio \
		-D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors

# 根据 BF 文件生成对应的代码
# 如果需要修改宏信息，可以在 generator 中修改
kernel/src/object/structures.rs: tools/aarch64/structures.bf tools/structures.bf tools/*.py tools/templates/*.rs.j2
	python3 tools/generator.py $< $@

bf: kernel/src/object/structures.rs

.PHONY: all run build clean fmt
