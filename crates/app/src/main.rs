//! App service (Part E). Currently a smoke test that wires the HAL mocks to
//! the game crate. Printing the OS/arch lets you confirm a cross-compiled
//! binary really is aarch64 when you run it on the Pi.

use pting_hal::mock::{MockHitSensor, MockLights};
use pting_hal::{Clock, HitSensor, Lights, Rgb, SystemClock, TargetId};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|a| a == "--version") {
        println!("pting-app {VERSION}");
        return Ok(());
    }
    println!(
        "pting-app {VERSION} ({}/{})",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let clock = SystemClock;
    let mut lights = MockLights::new(10);
    let mut sensor = MockHitSensor::new();
    let injector = sensor.injector();
    let hits = sensor.events()?;

    let target = TargetId(3);
    lights.set(target, Rgb(255, 0, 0))?;
    let lit_at = clock.now();

    // Stand-in for a real BB hit.
    injector.hit(target, clock.now(), 512);

    let hit = hits.recv()?;
    let ms = pting_game::reaction_ms(lit_at, &hit);
    println!("target {} hit, reaction {:?} ms", hit.target.0, ms);
    Ok(())
}
