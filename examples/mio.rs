extern crate zwave;

fn main() {
    zwave::run("/dev/ttyACM0").unwrap();
}
