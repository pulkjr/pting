After making changes to boot.cmd follow this:

1. Compile the script (mkimage lives in your U-Boot tree)

    ```bash
    ~/Projects/u-boot/tools/mkimage -A arm64 -T script -C none \
      -d image/uboot/boot.cmd build/qemu/boot.scr
    ```

2. Copy it onto the FAT partition of the disk (-o overwrites the old one)
    ```bash
    mcopy -o -i build/qemu/disk.img@@1M build/qemu/boot.scr ::
    ```

3. Boot
    ```bash
    qemu-system-aarch64 -machine virt -cpu cortex-a57 -nographic \
      -bios ~/Projects/u-boot/u-boot.bin \
      -drive if=none,file=build/qemu/disk.img,format=raw,id=hd0 \
      -device virtio-blk-device,drive=hd0`
    ```
