//use nanomsg::{Socket, NanoResult, Protocol};

use chrono::*;

use capnp::serialize_packed;
use capnp::{MessageBuilder, MallocMessageBuilder};
use capnp::io::WriteOutputStream;

use std::ops::*;
use std::io::*;

use std::thread;
use std::thread::JoinGuard;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{Receiver, SyncSender};

#[derive(Debug)]
#[allow(dead_code)]
pub enum DataValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Text(String),
}

struct RawDataPoint {
    location: String,
    path: String,
    component: String,
    timestamp: DateTime<UTC>,
    value: DataValue,
}


pub struct CollectorThread<'a> {
    thread: JoinGuard<'a, ()>,
    sink: SyncSender<Box<RawDataPoint>>,
}

impl<'a> CollectorThread<'a> {
    pub fn spawn() -> CollectorThread<'a> {
        let (tx, rx): (SyncSender<Box<RawDataPoint>>, Receiver<Box<RawDataPoint>>) = sync_channel(1000);

        let thread = thread::scoped(move || {
            match rx.recv() {
                Ok(raw_data_point) => {
                    let mut message = MallocMessageBuilder::new_default();
                    {
                        let mut raw_data_point_builder = message.init_root::<super::raw_data_point_capnp::raw_data_point::Builder>();

                        raw_data_point_builder.set_location(&*raw_data_point.location);
                        raw_data_point_builder.set_path(&*raw_data_point.path);
                        raw_data_point_builder.set_component(&*raw_data_point.component);

                        {
                            let mut date_time_builder = raw_data_point_builder.borrow().init_timestamp();
                            date_time_builder.set_unix_timestamp(raw_data_point.timestamp.timestamp());
                            date_time_builder.set_nanosecond(raw_data_point.timestamp.nanosecond());
                        }

                        {
                            let mut _value = raw_data_point_builder.borrow().init_value();
                            match raw_data_point.value {
                                DataValue::Integer(value) => {
                                    _value.set_integer(value);
                                },
                                DataValue::Float(value) => {
                                    _value.set_float(value);
                                },
                                DataValue::Bool(value) => {
                                    _value.set_boolean(value);
                                },
                                DataValue::Text(value) => {
                                    _value.set_text(&*value);
                                },
                            }
                        }
                    }

                    let mut out = WriteOutputStream::new(stdout());

                    serialize_packed::write_packed_message_unbuffered(&mut out, &mut message).ok().unwrap();
                },
                Err(error) => {
                    // TODO
                }
            }
        });

        CollectorThread {
            thread: thread,
            sink: tx
        }
    }

    pub fn new_collector(& self) -> Collector {
        Collector {
            timestamp: UTC::now(),
            sink: self.sink.clone(),
        }
    }
}

pub struct Collector {
    timestamp: DateTime<UTC>,
    sink: SyncSender<Box<RawDataPoint>>,
}

impl Collector {
    pub fn collect(& mut self, location: &str, path: &str, component: &str, value: DataValue) -> () {
        let raw_data_point = Box::new(RawDataPoint {
            location: location.to_string(),
            path: path.to_string(),
            component: component.to_string(),
            timestamp: self.timestamp,
            value: value
        });

        self.sink.send(raw_data_point);
    }
}

