#!/usr/bin/env bash
# Boots U-Boot in QEMU with slot B pre-broken and checks it falls back to A.
set -euo pipefail
cd "$(dirname "$0")/.."

UBOOT_DIR="${UBOOT_DIR:-$HOME/Projects/u-boot}"
WORK=build/qemu-test
DISK="$WORK/disk.img"
LOG="$WORK/console.log"

echo "Cleaning previous build in $WORK"
rm -rf "$WORK"; mkdir -p "$WORK"
truncate -s 64M "$DISK"
echo 'start=2048, type=c' | sfdisk -q "$DISK"
mformat -i "$DISK@@1M" -F ::

"$UBOOT_DIR/tools/mkimage" -A arm64 -T script -C none \
    -d image/uboot/boot.cmd "$WORK/boot.scr" >/dev/null

echo "Seeding env: prefer slot B (which always fails), 3 attempts each."
cat > "$WORK/seed.txt" <<'EOF'
bootdelay=0
scriptaddr=0x40200000
bootcmd=load virtio 0:1 ${scriptaddr} boot.scr && source ${scriptaddr}
BOOT_ORDER=B A
BOOT_A_LEFT=3
BOOT_B_LEFT=3
EOF

"$UBOOT_DIR/tools/mkenvimage" -s 262144 -o "$WORK/uboot.env" "$WORK/seed.txt"
mcopy -i "$DISK@@1M" "$WORK/boot.scr" "$WORK/uboot.env" ::

echo "starting qemu"
qemu-system-aarch64 -machine virt -cpu cortex-a57 \
    -display none -monitor none -serial "file:$LOG" \
    -bios "$UBOOT_DIR/u-boot.bin" \
    -drive "if=none,file=$DISK,format=raw,id=hd0" \
    -device virtio-blk-device,drive=hd0 &
QPID=$!
trap 'kill "$QPID" 2>/dev/null || true' EXIT

echo "Waiting for QEMU (up to 2 min)..."
for _ in $(seq 1 120); do
    grep -q 'SIM: slot A booted OK' "$LOG" 2>/dev/null && break
    kill -0 "$QPID" 2>/dev/null || break
    sleep 1
done
kill "$QPID" 2>/dev/null || true
wait "$QPID" 2>/dev/null || true

got="$(grep -o 'Trying slot [AB]' "$LOG" | tr '\n' ',')"
want="Trying slot B,Trying slot B,Trying slot B,Trying slot A,"
if [[ "$got" == "$want" ]]; then
    echo "PASS: B tried 3 times, then fell back to A"
else
    echo "FAIL: attempt sequence was: ${got:-<none>}" >&2
    echo "expected:                   $want" >&2
    echo "--- console log tail ---" >&2
    tail -40 "$LOG" >&2
    exit 1
fi
