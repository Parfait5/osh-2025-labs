# 实验一

### Linux 编译

在 x86_64 电脑 上交叉编译 ARM64（aarch64）Linux 内核，然后用 QEMU 运行。首先安装ARM64交叉编译器和对应版本的qemu作为ARM64模拟器
```bash
sudo apt update
sudo apt install -y gcc-aarch64-linux-gnu make bc bison flex libssl-dev qemu-system-aarch64
```
随后配置内核，采用默认设置,
```bash
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- defconfig
```
然后编译
```bash
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j$(nproc)
```
然后创建一个临时根文件系统，在qemu上运行Linux内核
```bash
mkdir -p rootfs/{bin,sbin,etc,proc,sys,usr/bin,usr/sbin}
cd rootfs
find . | cpio -o --format=newc | gzip > ../initramfs.cpio.gz
qemu-system-aarch64 -machine virt -cpu cortex-a57 -m 1024 \
    -kernel arch/arm64/boot/Image \
    -initrd initramfs.cpio.gz \
    -append "console=ttyAMA0" \
    -nographic
```

---
### 创建初始内存盘



---
### 添加自定义系统调用