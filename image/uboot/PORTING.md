# Porting the QEMU boot logic to the Raspberry Pi

Status: notes written at the end of Part A. Part B turns these into real work.

## What QEMU proved
- Env persists in `uboot.env` on a FAT partition (`saveenv` / reload).
- RAUC-style slot selection: `BOOT_ORDER`, `BOOT_A_LEFT`, `BOOT_B_LEFT`.
- Attempts are decremented and saved *before* booting, so a crash keeps the count.
- A slot with 0 attempts is skipped; when none are left, counters reset and the board resets.
- LOADER-style commands: `boot_image1`, `boot_image2`, `boot_primary`, `boot_backup`, `autoboot` (run with `run <name>`).
- `tools/qemu-test-fallback.sh` checks the B,B,B,A sequence.

## What QEMU does NOT prove
- The Pi firmware to U-Boot handoff, SD card behavior, real kernel hangs, power loss.

## Differences to handle in Part B

| Topic | QEMU | Pi |
|---|---|---|
| Defconfig | `qemu_arm64_defconfig` | `rpi_arm64_defconfig` (expected; confirm for Pi 3 and Zero 2 W) |
| Env storage | FAT on `virtio 0:1` | FAT file on the SD boot partition (`mmc 0:1`) |
| Env size | 256 KB default | Set explicitly, much smaller (e.g. 16 KB) |
| Redundant env (power-loss safety) | not used | Check whether `ENV_IS_IN_FAT` supports redundancy in our U-Boot version. If not, consider keeping a second copy or accepting the risk. |
| Loading U-Boot | `-bios u-boot.bin` | Pi firmware loads it (`kernel=u-boot.bin` and `arm_64bit=1` in `config.txt`) |
| Device tree | provided by QEMU | The Pi firmware passes its own. Zero 2 W and Pi 3 differ. Verify per board. |
| Slot boot | simulated by `sim_boot_A/B` | Load kernel (+ DTB) from the slot, set `root=` to the slot's rootfs partition, `booti` |

## Failed boots must end in a reset
A hung or panicking kernel never returns control to U-Boot, so the attempt counter only helps if the board actually reboots:
- Kernel command line: `panic=5` (or similar) so panics reboot.
- Enable the SoC hardware watchdog and have userspace feed it. If the app or init stops, the board resets.
- Verify: deliberately hang a test image and confirm the board reboots and falls back.

## Confirming a good boot
The booted OS must reset its slot's attempts counter (RAUC does this through `fw_setenv`/`rauc status mark-good`). That needs `fw_env.config` on the target pointing at the FAT env file. Plan this in Part B and test it in Part H.

## Before trusting the Pi port
- Power-loss test during `saveenv`.
- Slot B attempts exhausted, then confirm the device stays on A across power cycles.
- Both slots exhausted: confirm the device recovers by resetting the counters rather than looping forever without booting.
