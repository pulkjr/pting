# Pting

A wall-mounted, 1v1 airsoft target game powered by a Raspberry Pi. Each target is a steel can mounted in a wooden face with an LED behind it. The Pi lights a target, starts a timer, and listens for the BB to hit the can. Reaction time becomes the score. Players connect from their phones over the Pi's own WiFi.

> **Status:** Planning. Nothing is built yet. See [docs/PLAN.md](docs/PLAN.md).

## Safety

- **Everyone in the room wears eye protection (ANSI Z87.1 rated or better).** No exceptions.
- Steel cans can bounce BBs back toward the shooter. Use a baffle or soft backstop inside and behind the cans, and a way to empty them.
- Keep LEDs and wiring away from the can bottoms where BBs land.
- Cap LED brightness in software and use a properly rated power supply.

## What it is

| Piece | Description |
|---|---|
| **Hardware** | Wood face with a 2×5 grid of equal holes (configurable). Each hole has a steel can, a piezo hit sensor, and a WS2812 LED. |
| **Boot loader** | U-Boot with an ONTAP LOADER-style command layer (`printenv`, `setenv`, `boot_image1`, `boot_primary`, ...). |
| **OS image** | Minimal Buildroot Linux with two A/B slots, a read-only rootfs, and a persistent data partition. |
| **App** | Rust service: hardware abstraction, game engine, REST API, WiFi AP, scheduled self-update. |
| **Updates** | Signed bundles published to GitHub Releases. The device downloads to the inactive slot, switches, and reverts automatically if the new image fails to boot. |

## Hardware

- **Dev board:** Raspberry Pi 3 (owned)
- **Target board:** Raspberry Pi Zero 2 W
- **Debug aid:** Raspberry Pi Pico as a USB-to-UART adapter (debug probe firmware)
- **Sensing:** piezo disc per can, read through MCP3008 ADCs (SPI)
- **Lights:** WS2812 per hole, via SPI with a 74AHCT125 level shifter
- **Dev machine:** x86_64 Linux (Omarchy on a T2 MacBook Pro)

## Repository layout (planned)

```
airsoft-arena/
├── README.md
├── docs/
│   ├── PLAN.md          # project plan, shared context, per-part briefs
│   └── decisions/       # short ADRs as we make choices
├── crates/
│   ├── hal/             # hardware traits + real (Pi) and mock implementations
│   ├── game/            # pure-logic game engine (no hardware dependency)
│   └── app/             # REST server, config, provisioning, updater
├── image/               # Buildroot external tree, U-Boot env/scripts, RAUC config
├── web/                 # phone-friendly client UI
├── tools/               # bench programs, QEMU test scripts, release helpers
└── .github/workflows/   # CI: build, test, sign, release
```

## Getting started

Nothing to run yet. Start with [docs/PLAN.md](docs/PLAN.md), pick a part, and follow its brief.

## Working with an AI agent

Each part of the plan is written so it can be worked on in isolation. Copy **Section 0 (Shared Context)** plus the one Part you are working on into a new session, along with the prompt template in Section 3. When a part is done, update its status and log any decisions in `docs/decisions/`.

## License

TBD
