# Hardware parts list

## Electronics

| Part | Qty | Notes |
|---|---|---|
| MCP3008-I/P (DIP-16) | 2 | 8-channel SPI ADC. One for the spike, a second for the ten-can build (2×8 = 16 channels). |
| 27 mm piezo disc elements, bare | 10+ | One per can plus spares, since they crack. |
| 1 MΩ resistors | 10+ | Across each piezo so it discharges. |
| Resistor assortment kit | 1 | Covers the 1k-10k series resistors and the 330 Ω LED data resistor. |
| Small-signal Schottky diodes, BAT85 or BAT43 (1N5819 works) | 20 | Two per channel, clamping the input to 0-3.3 V. |
| WS2812B strip, 60 LED/m, non-waterproof, 1 m | 1 | Cut into single pixels. Use 3-5 for the spike, ten for the full build. |
| 74AHCT125N (DIP-14) | 2 | Level-shifts the Pi's 3.3 V data to 5 V for the LEDs. |
| 1000 µF, 10 V or higher electrolytic capacitor | 1 | Across the LED supply, standard WS2812 practice. |
| 5 V, 4 A supply with a barrel jack, plus a screw-terminal adapter | 1 | LEDs get their own supply, sharing ground with the Pi. |
| Breadboard and jumper wires (M-M and M-F) | 1 set | |

## Mechanical and safety

| Part | Notes |
|---|---|
| Steel vegetable can | Free. Label off, cleaned, open at the top. |
| Foam tape or rubber weatherstripping | Isolates the can from the wood. |
| Epoxy or cyanoacrylate glue | Piezo to can. Try double-sided tape too, since bond stiffness affects sensitivity. |
| Airsoft BBs, 0.20 g | Plus whatever gun you own. |
| **ANSI Z87.1 safety glasses** | One pair per person in the room. |
| Backstop | A cardboard box packed with rags or foam behind the can. |

## Optional

- 8-channel USB logic analyzer (about $10) for validating timing
- Multimeter, if you don't have one

## Not yet

- Two Raspberry Pi Zero 2 W boards

## Notes

- The wiring plan and bench program come with the Part C session.
- Give the LEDs their own 5 V supply and share ground with the Pi.
- Piezos can spike well above 3.3 V, so the series resistor and clamp diodes are required on every channel.
