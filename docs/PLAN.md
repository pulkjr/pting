# Project Plan

Last updated: 2026-10-18

Contents:
0. Shared Context (the spec)
1. Phases and milestones
2. Parts A-H (self-contained briefs)
3. Session prompt template
4. Risks
5. Decision log
6. Status tracker

---

# 0. Shared Context

> **Copy this whole section, plus one Part from Section 2, into a new AI session.**

## 0.1 Goal

Build a framework to develop, test, release, and auto-update a custom OS for a Raspberry Pi that runs an airsoft target game. The user is new to embedded Linux and is working with an AI agent as a pair engineer. Explain the *why* behind steps, prefer small verifiable steps, and always say how to test the result.

## 0.2 Spec

1. **Boot loader.** A loader with a small command set, similar to NetApp ONTAP's LOADER:
   - `printenv`, `setenv`, `saveenv` to show and set boot arguments
   - `autoboot`, `boot_image1`, `boot_image2`, `boot_primary`, `boot_backup`
2. **Two image slots.** Download a new image into the backup slot, mark it primary, and reboot. If the boot fails, the loader automatically boots the previous image.
3. **OS image.** A simple custom Linux that abstracts the hardware, so the Rust app can run in a container on the developer's machine against a mock/test harness for the attached hardware.
4. **App.** A Rust app that reads hit sensors and drives lights. It exposes a REST server for a custom app or website. It auto-updates from a given website on a schedule.
5. **Game.** The Pi is the game brain. A player connects to the Pi's WiFi (or Bluetooth) from a phone, selects 1v1 mode, and picks a color for each player. The device is a wood face with equally sized, equally spaced holes in a grid. When the game starts, a target lights in the selected color and a timer starts. The player shoots it. The device detects the hit, and the time from light-on to detected hit becomes the shot's score. Turns alternate between player A and player B until one reaches the winning score.

## 0.3 Decisions so far

| Item               | Decision                                                                                                                                                                        |
|--------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Target board       | Raspberry Pi Zero 2 W (aarch64, 512 MB RAM, WiFi/BT)                                                                                                                            |
| Dev board          | Raspberry Pi 3 (same Cortex-A53 family). Board is a build variable: `BOARD=rpi3 \| rpizero2w`                                                                                   |
| Boot chain         | Pi ROM and GPU firmware (closed, unchangeable), then **U-Boot**, then Linux. The "loader" is U-Boot plus a LOADER-style command layer.                                          |
| Fallback mechanism | U-Boot `bootcount` / `bootlimit` / `altbootcmd`                                                                                                                                 |
| OS build system    | Buildroot, run inside a Podman/Docker container                                                                                                                                 |
| Update mechanism   | RAUC (or SWUpdate, to be confirmed in Part A/H) for signed bundles and atomic slot writes. The Rust app is the scheduler/downloader.                                            |
| Update host        | GitHub Releases (public repo). Stable URL: `releases/latest/download/manifest.json`. Beta channel via a rolling tag such as `channel-beta`.                                     |
| Language           | Rust, cross-compiled to `aarch64-unknown-linux-gnu`                                                                                                                             |
| Projectile         | Airsoft 6 mm BBs                                                                                                                                                                |
| Targets            | 10 holes (2×5), configurable `N`. Each is a steel can on the back of a wood face.                                                                                               |
| Hit sensing        | One piezo disc per can, read through MCP3008 ADC(s) over SPI. Replaces the original single-microphone idea, because a mic hears the gun shot and cannot tell which can was hit. |
| Lights             | One WS2812 per hole, over SPI with a 74AHCT125 level shifter, mounted on the back of the wood or rim, not the can bottom                                                        |
| Dev machine        | x86_64 Linux (Omarchy on a T2 MacBook Pro). QEMU runs under software emulation.                                                                                                 |
| Wireless           | The Pi hosts a WiFi access point. Bluetooth is a stretch goal.                                                                                                                  |

## 0.4 Assumptions to revisit

- Hits on the wooden face should read as misses. Cans are isolated with rubber or foam.
- Hits on a can other than the lit target are ignored (the alternative is a penalty). Decide in Part F.
- Human reaction time is in the hundreds of ms, so ±5-10 ms timing accuracy is enough.
- Piezos can spike above 3.3 V, so protection (series resistor and clamp diodes) is required.

## 0.5 Interface contracts

These are the seams between parts. Changing one means updating every part that touches it.

### Boot environment variables (Parts A, B, H) - provisional

| Variable                     | Meaning                                                                                   |
|------------------------------|-------------------------------------------------------------------------------------------|
| `BOOT_ORDER`                 | Space-separated slot order, e.g. `A B`. First entry is primary, second is backup.         |
| `BOOT_A_LEFT`, `BOOT_B_LEFT` | Boot attempts remaining for each slot. Reset to 3 by the OS when it confirms a good boot. |
| `boot_slot`                  | Slot to boot next: `a` or `b` (the "primary")                                             |
| `slot_a_ok`, `slot_b_ok`     | Whether each slot has been confirmed healthy                                              |
| `upgrade_available`          | `1` while a newly written slot is on trial                                                |
| `bootcount`                  | Incremented by U-Boot each boot attempt                                                   |
| `bootlimit`                  | Max attempts before `altbootcmd` runs                                                     |

*Note:* RAUC's native U-Boot integration uses `BOOT_ORDER` and `BOOT_A_LEFT` / `BOOT_B_LEFT`. Part A must decide whether to adopt those names instead. The names above are provisional until then.

### LOADER-style commands (Part A)

`printenv`, `setenv`, `saveenv`, `autoboot`, `boot_image1` (slot A), `boot_image2` (slot B), `boot_primary` (slot in `boot_slot`), `boot_backup` (the other slot).

### Update flow (Parts A, B, H)

1. App checks the manifest on its schedule and downloads a signed bundle.
2. The bundle is written to the inactive slot (via RAUC).
3. Boot variables are set so the next boot uses the new slot, with `upgrade_available=1` and `bootcount=0`. Reboot.
4. New image boots. After the app passes its health check, it clears `upgrade_available` and marks the slot good.
5. If `bootcount` exceeds `bootlimit`, `altbootcmd` flips back to the previous slot.

### HAL traits (Parts D, E, F) - sketch

```rust
pub struct TargetId(pub u8);
pub struct Rgb(pub u8, pub u8, pub u8);

pub struct HitEvent {
    pub target: TargetId,
    pub at: std::time::Instant, // monotonic timestamp of detection
    pub peak: u16,              // raw sensor magnitude, for tuning
}

pub trait Lights {
    fn set(&mut self, target: TargetId, color: Rgb) -> Result<()>;
    fn clear(&mut self, target: TargetId) -> Result<()>;
    fn clear_all(&mut self) -> Result<()>;
}

pub trait HitSensor {
    /// Async stream (or channel receiver) of hit events.
    fn events(&mut self) -> HitStream;
}

pub trait Clock {
    fn now(&self) -> std::time::Instant;
}
```

Implementations: `real` (Pi hardware) and `mock` (scripted or keyboard-driven, used in containers and tests).

### Game events (Parts E, F, G)

`GameCreated`, `PlayerJoined`, `ColorsChosen`, `GameStarted`, `TurnStarted { player }`, `TargetLit { target, color }`, `HitDetected { target, reaction_ms }`, `TurnTimedOut`, `RoundScored { player, points }`, `GameWon { player }`.

### REST API (Parts E, G)

To be defined as an OpenAPI file at `crates/app/openapi.yaml`. Expected resources: device info, WiFi config, game create/join/start/state, live events (SSE or WebSocket), update status and manual check.

### Update manifest (Part H)

JSON with `version`, `channel`, `bundle_url`, `sha256`, `min_version`, `released_at`. Bundle signatures are verified by RAUC.

## 0.6 Glossary

- **Slot:** one of two complete bootable OS images (A and B).
- **Primary / backup:** the slot that boots next / the other slot.
- **A/B update:** write the new OS to the inactive slot, switch, and keep the old one as a fallback.
- **Bundle:** a signed update file (RAUC `.raucb`).
- **HAL:** hardware abstraction layer, the Rust traits that hide real vs mock hardware.
- **Buildroot:** a tool that builds a minimal Linux image from source.
- **U-Boot:** the bootloader that runs after the Pi's firmware.

---

# 1. Phases and milestones

| Phase | Name                  | Parts              | Exit criteria                                                                                                                                                                                              |
|-------|-----------------------|--------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 0     | Foundations           | -                  | Repo skeleton, Rust workspace builds, cross-compile to aarch64 works, container dev setup documented                                                                                                       |
| 1     | Three parallel tracks | C, D+F, A          | **C:** bench program lights a target and prints reaction time from a real BB hit. **D+F:** full 1v1 game playable in a terminal with mock hardware. **A:** in QEMU, a forced-bad slot B auto-reverts to A. |
| 2     | Real image            | B (+A on hardware) | Boots from SD on real hardware. `printenv`, `boot_image2`, and bad-image fallback all work. Serial console available.                                                                                      |
| 3     | Integration           | E, G               | Phone joins the Pi's WiFi AP and plays a full game on real hardware                                                                                                                                        |
| 4     | OTA                   | H                  | Push a release to GitHub, the device updates itself and survives a deliberately broken release                                                                                                             |
| 5     | Field testing         | -                  | Scoring tuned, noise rejection verified, enclosure built, power-loss soak test passed                                                                                                                      |

**Why this order:** hit detection (Part C) is the largest technical risk and the game depends on it, so it is validated first. The game engine (Part F) is pure logic and can start immediately. Part A can be fully tested in QEMU without hardware.

---

# 2. Parts

Each part below is a self-contained brief. Copy Section 0 plus one part into a new session.

---

## Part A - Boot & Slots

**Goal:** U-Boot configured for the A/B scheme with LOADER-style commands and automatic fallback.

**Scope**
- Partition layout: boot firmware, U-Boot env, slot A (kernel + rootfs), slot B, persistent data
- U-Boot environment and boot scripts implementing the variables in 0.5
- LOADER-style commands as U-Boot scripts/aliases: `autoboot`, `boot_image1`, `boot_image2`, `boot_primary`, `boot_backup`
- `bootcount` / `bootlimit` / `altbootcmd` fallback logic
- Decide: native RAUC variable names (`BOOT_ORDER`, `BOOT_x_LEFT`) vs custom names
- A QEMU (`qemu-system-aarch64`, `virt`) test harness so the logic can be tested without hardware

**Out of scope:** the Pi-specific firmware and device tree (Part B), update bundles (Part H)

**Depends on:** none

**Tasks**
1. Build U-Boot for `qemu_arm64` in a container and get a console in QEMU.
2. Write the env layout and boot scripts. Store the env somewhere persistent (a file/partition in QEMU).
3. Implement the LOADER-style commands as scripts.
4. Implement the fallback logic. Test: mark slot B bad, boot it, and confirm automatic revert to A after `bootlimit` attempts.
5. Write a script `tools/qemu-test-fallback.sh` that runs the above and exits non-zero on failure.
6. Document how to port to the Pi (`rpi_3` / `rpi_arm64` U-Boot targets, env stored on the FAT boot partition).

**Exit criteria:** the QEMU fallback test passes in a script. All LOADER-style commands work from the U-Boot prompt. Variable names are finalized and recorded in 0.5.

**Open questions:** RAUC-native vs custom variable names. Where the env lives on the Pi (FAT file vs raw SD offset).

**Notes:** QEMU runs under software emulation on x86, so it will be slow. That is fine for this test.

---

## Part B - OS Image Build

**Goal:** A reproducible minimal Linux image for the Pi 3 and Zero 2 W with two root slots and a data partition.

**Scope**
- Buildroot external tree in `image/` with per-board configs (`rpi3`, `rpizero2w`)
- Containerized build (Podman/Docker) with a shared download cache and `ccache`
- Read-only root filesystem, persistent `/data` partition
- Init system, networking, WiFi access point (hostapd, dnsmasq or similar), SSH for development
- Serial console (UART, or USB gadget mode on the Zero 2 W)
- RAUC installed and configured for the A/B layout
- SD card image generation (`genimage`)

**Out of scope:** the Rust app itself (Part E), release signing/CI (Part H)

**Depends on:** Part A (boot env and partition layout)

**Tasks**
1. Containerized Buildroot build for `rpi3`. Boot to a login prompt.
2. Add U-Boot and the Part A layout. Boot slot A and slot B.
3. Add the persistent data partition and read-only root.
4. Add WiFi AP and verify a phone can join.
5. Add the `rpizero2w` variant (same layout, different config and device tree).
6. Document the build in `image/README.md`, including disk space needs (30+ GB) and first-build time.

**Exit criteria:** `make BOARD=rpi3` produces a flashable SD image that boots both slots, serves a WiFi AP, and survives a power cycle with `/data` intact. Same for `rpizero2w`.

**Open questions:** systemd vs a lighter init (BusyBox init). How the Rust binary gets into the image (Buildroot package vs prebuilt artifact).

---

## Part C - Hardware Bring-up

**Goal:** Prove the sensors and lights work, and measure hit-detection quality, before anything else depends on them.

**Scope**
- Wiring and a parts list for one-can and then ten-can setups
- WS2812 LEDs over SPI with a 5 V level shifter
- Piezo sensing through MCP3008 with protection (series resistor, clamp diodes)
- Bench programs (any language) run on stock Raspberry Pi OS Lite on the Pi 3
- Measurements: detection latency, false-trigger rate (gun shot noise, bumping the wood, talking), sensitivity vs BB velocity

**Out of scope:** integration with the HAL or app (Part D/E), the custom OS

**Depends on:** none

**Tasks**
1. Spike hardware (about $20): one can, one piezo disc, one MCP3008, a few WS2812s.
2. Wire LED + level shifter. Light a color from a bench program.
3. Wire piezo + protection + ADC. Print raw samples while tapping the can.
4. Implement threshold detection. Fire BBs at the can and log hits.
5. Measure: sample rate achieved, time from impact to detection, false triggers from firing the gun near it, and rejection of wood-face hits.
6. Write results into `docs/decisions/` (sensor choice, threshold method, mounting/isolation approach).
7. Scale to 10 channels (two MCP3008s) and re-measure.

**Exit criteria:** a bench program lights a target, waits for a hit, and prints reaction time. False-trigger rate and timing accuracy are documented. Wiring diagram is committed.

**Open questions:** ADC polling vs comparator + GPIO interrupt. Required sample rate. Mounting method that isolates cans from the wood.

**Safety:** eye protection for everyone, baffles and backstop, see README.

---

## Part D - HAL & Dev Harness

**Goal:** Rust hardware traits with real and mock implementations, so everything above the HAL runs in a container on a laptop.

**Scope**
- `crates/hal`: `Lights`, `HitSensor`, `Clock` traits (see 0.5)
- Mock implementation: scripted hits, keyboard-driven hits, and a simulated clock for deterministic tests
- Real implementation: `spidev` for WS2812 and MCP3008 (informed by Part C results)
- Config for `N` targets and channel mapping
- Dev container to run the app against the mock
- Cross-compile setup for `aarch64-unknown-linux-gnu`

**Out of scope:** game rules (Part F), REST (Part E)

**Depends on:** none (real impl uses Part C findings)

**Tasks**
1. Create the Rust workspace and CI-friendly `cargo test`.
2. Define the traits and types. Keep them small.
3. Write the mock, including a way to inject hits with specific timestamps.
4. Write a tiny terminal harness: keys 0-9 simulate hits, the lights print as colored blocks.
5. Cross-compile and run the real implementation on the Pi 3 using what Part C proved.

**Exit criteria:** terminal harness works on the laptop. The same trait-based code compiles for aarch64 and drives real hardware.

**Open questions:** async runtime (tokio) vs threads. Hit stream design (channel vs `Stream`).

---

## Part E - App Core

**Goal:** The Rust service that ties hardware, game, and network together.

**Scope**
- `crates/app`: config, device state, logging
- REST API with an OpenAPI spec, plus a live event channel (SSE or WebSocket)
- WiFi provisioning hooks (AP mode by default, optional join to home WiFi)
- Health check endpoint and logic used by the update flow (Part H)
- Systemd/init service definition

**Out of scope:** the UI (Part G), updater logic (Part H), game rules (Part F)

**Depends on:** Part D (traits), Part F (engine)

**Tasks**
1. Choose the web framework (axum is the likely choice) and write `openapi.yaml` first.
2. Implement device info, game create/join/start/state, and event streaming.
3. Wire the HAL and game engine together.
4. Add the health check: "healthy" means the app started, the HAL initialized, and the API answers.
5. Run it in the container against the mock. Run it on the Pi against real hardware.

**Exit criteria:** `curl` can create and play a full game via the API in the container, then on the Pi.

---

## Part F - Game Engine

**Goal:** A pure-logic state machine for the game with no hardware dependency.

**Scope**
- `crates/game`: modes (1v1 now, more players later), players, colors, turns, scoring, win condition
- Event model from 0.5
- Scoring formula, timeouts, and rules for wrong-can hits
- Deterministic tests using a simulated clock

**Out of scope:** hardware, networking

**Depends on:** none (uses D's types)

**Tasks**
1. Model the state machine: `Lobby → ColorsChosen → TurnActive → Scored → ... → Finished`.
2. Pick a scoring formula (for example, faster reaction = more points, with a max per shot) and a win condition (first to X points, or best of N).
3. Decide and implement: wrong-can hit (ignore vs penalty), timeout, early/duplicate hits, random target selection with no immediate repeats.
4. Property and unit tests with a fake clock.
5. Terminal game using the mock HAL (shared with Part D).

**Exit criteria:** a full 1v1 game plays in a terminal with scripted and keyboard hits. Tests cover scoring, timeouts, and win conditions.

**Open questions:** the scoring formula. Simultaneous multi-player mode with more holes (design the engine so it is not hard-coded to 2 players).

---

## Part G - Client UI

**Goal:** A phone-friendly web interface served by the Pi.

**Scope**
- `web/`: small PWA served by the app
- Screens: connect, choose 1v1, pick colors, live game view, scores, settings
- Uses only the REST/event API from Part E

**Out of scope:** native apps, Bluetooth (stretch)

**Depends on:** Part E (API), Part F (events)

**Tasks**
1. Generate or hand-write an API client from `openapi.yaml`.
2. Build screens against the container-hosted mock app first.
3. Handle captive-portal behavior so the phone opens the UI after joining the AP.
4. Test on a real phone against the Pi.

**Exit criteria:** from a phone, join the Pi's AP and play a full game.

**Open questions:** framework (plain JS, Svelte, etc.). Whether to embed the build output into the Rust binary.

---

## Part H - Update & Release

**Goal:** Signed A/B updates from GitHub Releases with automatic rollback.

**Scope**
- RAUC (or SWUpdate) bundle creation, signing keys, and certificate handling
- CI pipeline: build image, run tests, build the bundle, sign, publish a GitHub Release with `manifest.json`
- On-device updater (inside the Rust app): scheduled manifest check, download, verify, install to the inactive slot, set boot variables, reboot
- Health-check confirmation after reboot
- Release channels (stable via `latest`, beta via a rolling `channel-beta` tag)
- Safety: don't update mid-game, check free space, resumable download, version rules

**Out of scope:** U-Boot internals (Part A), image contents (Part B)

**Depends on:** Parts A, B, E

**Tasks**
1. Generate a dev signing key and cert. Store the private key outside the repo and use CI secrets for releases.
2. Build a RAUC bundle locally and install it by hand on the device.
3. Write the updater in the app. Test against a local HTTP server first.
4. Add the CI release workflow.
5. Test failure paths: corrupt download, bad signature, power loss mid-install, a release that boots but fails its health check, and one that doesn't boot at all.

**Exit criteria:** publish a release and the device updates itself. A deliberately broken release rolls back automatically and the device stays usable.

**Open questions:** RAUC vs SWUpdate (confirm in Part A). Update schedule and quiet hours. Key rotation plan.

---

# 3. Session prompt template

Paste this at the top of a new AI session, followed by Section 0 and the Part you're working on:

```
You are an expert in embedded Linux and Rust, acting as my pair engineer on this
project. I am new to embedded Linux, so explain why each step matters, keep steps
small and testable, and tell me how to verify each one. Below is the shared
project context (Section 0) and the brief for the part I want to work on today.

Constraints:
- Stay inside this part's scope. If something requires changing an interface in
  Section 0.5, stop and tell me instead of silently changing it.
- Ask before assuming hardware details I haven't given you.
- Give commands for an x86_64 Linux dev machine (Omarchy/Arch), using containers
  for builds where practical.

Start by restating the goal of this part, then propose the first 3 steps.

[paste Section 0]
[paste Part X]
```

When the session ends, ask the agent for a summary of decisions and changed interfaces, then update `docs/decisions/` and Section 0.5 here.

---

# 4. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Piezo hit detection is noisy or unreliable | Game doesn't work | Part C first. Isolate cans from the wood. Tune thresholds in software via ADC. Fall back to a different sensor if needed. |
| Gun-shot noise causes false triggers | Wrong scores | Piezo (contact) not mic. Add a short blanking window and a threshold-plus-duration filter. |
| Timing jitter from Linux scheduling | Unfair scores | Measure in Part C. Timestamp at sample time. Consider a real-time priority thread or a Pico co-processor later. |
| U-Boot on Pi is fiddly | Delays Parts A/B | Prove the logic in QEMU first. Keep the Pi port as a separate step. |
| Bad update bricks the device | Field failure | A/B + bootcount rollback + health check. Test failure paths explicitly in Part H. |
| Buildroot builds are slow or break on Arch | Frustration | Containerized builds, `ccache`, shared download cache. |
| BBs bounce back / injuries | Safety | Eye protection rule, baffles, backstop. See README. |
| Signing key loss or leak | Can't update, or unsafe updates | Offline backup of the private key, CI secrets, documented rotation. |
| Zero 2 W has only 512 MB RAM | Memory pressure | Develop on the Pi 3 but monitor memory use. Test on the Zero 2 W by Phase 2. |

---

# 5. Decision log

Record new decisions in `docs/decisions/NNNN-title.md`. Summary:

| # | Date | Decision |
|---|---|---|
| 1 | 2026-10-18 | Target Zero 2 W, develop on Pi 3, board is a build variable |
| 2 | 2026-10-18 | U-Boot chained after Pi firmware provides the loader |
| 3 | 2026-10-18 | Buildroot (containerized) for the OS image |
| 4 | 2026-10-18 | Piezo-per-can sensing via MCP3008 replaces the single mic |
| 5 | 2026-10-18 | WS2812 LEDs over SPI with a level shifter |
| 6 | 2026-10-18 | GitHub Releases hosts updates (public repo) |
| 7 | 2026-10-18 | RAUC (or SWUpdate) for install mechanics; the app only schedules and downloads |

---

# 6. Status tracker

| Part                  | Status      | Owner | Notes |
|-----------------------|-------------|-------|-------|
| Phase 0 foundations   | Not started |       |       |
| A - Boot & Slots      | Not started |       |       |
| B - OS Image          | Not started |       |       |
| C - Hardware Bring-up | Not started |       |       |
| D - HAL & Harness     | Not started |       |       |
| E - App Core          | Not started |       |       |
| F - Game Engine       | Not started |       |       |
| G - Client UI         | Not started |       |       |
| H - Update & Release  | Not started |       |       |
