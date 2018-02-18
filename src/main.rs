#[macro_use]
extern crate log;
extern crate env_logger;
extern crate dbus;

mod dbus_bluez;
mod device;

fn run<'a>() -> Result<(), Box<std::error::Error>> {
    let dbus = dbus_bluez::DbusBluez::new()?;
    dbus.initialize()?;
    dbus.get_devices()?;
    Ok(())
}

fn main() {
    env_logger::init();
    ::std::process::exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            error!("{:?}", e);
            1
        },
    });
}
