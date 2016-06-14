extern crate zwave;
extern crate serial;

use std::env;
use std::io;
use std::time::Duration;

use std::io::prelude::*;
use serial::prelude::*;

mod frame_type {
    pub const MSG : u8 = 0x01;
    pub const ACK : u8 = 0x06;
    pub const NACK : u8 = 0x15;
    pub const CAN : u8 = 0x18;
}

fn main() {
    for arg in env::args_os().skip(1) {
        println!("{:?}", arg);
        let mut port = serial::open(&arg).unwrap();
        interact(&mut port).unwrap();
    }
}

#[derive(Debug)]
enum MsgError<'a> {
    ShortMsg,
    InvalidChecksum,
    UnknownType{data: &'a [u8]},
}

#[derive(Debug)]
enum Msg<'a> {
    Request(RequestMsg<'a>),
    Response(ResponseMsg<'a>),
}

impl<'a> Msg<'a> {
    fn new(buf : &'a [u8]) -> Result<Msg<'a>, MsgError> {
        if buf.len() < 2 {
            return Err(MsgError::ShortMsg);
        }
        let msg = &buf[..buf.len()-1];  // last byte is checksum.
        let checksum = msg.iter().fold(0xff ^ buf.len() as u8, |acc, byte| {
            acc ^ byte
        });
        if checksum != buf[buf.len()-1] {
            return Err(MsgError::InvalidChecksum);
        }
        match buf[0] {
            0 => Ok(Msg::Request(try!(RequestMsg::new(&msg[1..])))),
            1 => Ok(Msg::Response(try!(ResponseMsg::new(&msg[1..])))),
            _ => Err(MsgError::UnknownType{data: msg}),
        }
    }
}

#[derive(Debug)]
enum RequestMsg<'a> {
    ApplicationUpdate(ApplicationUpdateMsg<'a>),
}

impl<'a> RequestMsg<'a> {
    fn new(buf : &'a [u8]) -> Result<RequestMsg<'a>, MsgError> {
        if buf.len() < 1 {
            return Err(MsgError::ShortMsg);
        }
        let req_type = buf[0];
        match req_type {
            0x49 => Ok(RequestMsg::ApplicationUpdate(try!(ApplicationUpdateMsg::new(&buf[1..])))),
            _ => Err(MsgError::UnknownType{data: buf}),
        }
    }
}

#[derive(Debug)]
enum ApplicationUpdateMsg<'a> {
    InfoReceived{node_id : u8, rest: &'a [u8]},
}

impl<'a> ApplicationUpdateMsg<'a> {
    fn new(buf : &'a [u8]) -> Result<ApplicationUpdateMsg, MsgError> {
        if buf.len() < 1 {
            return Err(MsgError::ShortMsg);
        }
        let update_type = buf[0];
        match update_type {
            0x84 => Ok(ApplicationUpdateMsg::InfoReceived{node_id: buf[1], rest: &buf[2..]}),
            _ => Err(MsgError::UnknownType{data: buf}),
        }
    }
}

#[derive(Debug)]
struct ResponseMsg<'a> {
    msg : &'a [u8],
}

impl<'a> ResponseMsg<'a> {
    fn new(buf : &'a [u8]) -> Result<ResponseMsg<'a>, MsgError> {
        Err(MsgError::UnknownType{data: buf})
    }
}

fn interact<T: SerialPort>(port: &mut T) -> io::Result<()> {
    try!(port.reconfigure(&|settings| {
        try!(settings.set_baud_rate(serial::Baud115200));
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }));

    try!(port.set_timeout(Duration::from_millis(100000)));

    let mut buf: [u8; 256] = [0; 256];

    try!(port.read_exact(&mut buf[0..1]));
    let frame_type = buf[0];
    match frame_type {
        frame_type::MSG => {
            try!(port.read_exact(&mut buf[0..1]));
            let len = buf[0] as usize;
            let raw_frame = &mut buf[0..len];
            try!(port.read_exact(raw_frame));
            println!("raw_frame: {:?}", raw_frame);
            let msg = Msg::new(raw_frame).unwrap();
            println!("Received Msg: {:?}", msg);
        },
        frame_type::ACK => {
            println!("Received ACK");
        }
        frame_type::NACK => {
            println!("Received NACK");
        },
        frame_type::CAN => {
            println!("Received CAN");
        },
        _ => {
            println!("Received unknown frame");
        },
    }

    Ok(())
}
