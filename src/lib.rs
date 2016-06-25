extern crate mio;
extern crate serial;

use serial::SerialPort;
use std::ffi::OsStr;
use std::path::Path;
use std::os::unix::io::AsRawFd;

const MIO_TOKEN : mio::Token = mio::Token(0);

struct ZwaveHandler;

enum ZwaveState {
    ReadingMsg,
    WaitingForAck,
    WritingMsg,
    WritingAck,
}

impl mio::Handler for ZwaveHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut mio::EventLoop<ZwaveHandler>, token: mio::Token, events: mio::EventSet) {
        if events.is_readable() {
            println!("is readable");
        }

        if events.is_writable() {
            println!("is writeable");
        }
    }
}

pub fn run<T: AsRef<OsStr>>(path: T) -> Result<(), ()> {
    let mut port = serial::posix::TTYPort::open(Path::new(path.as_ref())).unwrap();
    port.reconfigure(&|settings| {
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }).unwrap();
    let mut event_loop = mio::EventLoop::new().unwrap();
    event_loop.register(&mio::unix::EventedFd(&port.as_raw_fd()), MIO_TOKEN, mio::EventSet::readable(), mio::PollOpt::level()).unwrap();
    let mut zwave = ZwaveHandler;
    event_loop.run(&mut zwave).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
