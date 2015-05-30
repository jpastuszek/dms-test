use super::serde::*;
use capnp::serialize_packed;
use capnp::{MessageBuilder, MallocMessageBuilder, MessageReader};
use capnp::message::ReaderOptions;
use capnp::io::ArrayInputStream;
use chrono::*;

#[derive(Debug)]
#[allow(dead_code)]
pub enum DataValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Text(String),
}

#[derive(Debug)]
pub struct RawDataPoint {
    pub location: String,
    pub path: String,
    pub component: String,
    pub timestamp: DateTime<UTC>,
    pub value: DataValue,
}

impl SerDeMessage for RawDataPoint {
    fn to_bytes(&self, encoding: Encoding) -> Result<Vec<u8>, SerializationError<Self>> {
        match encoding {
            Encoding::Capnp => {
                let mut message = MallocMessageBuilder::new_default();
                {
                    let mut raw_data_point_builder = message.init_root::<::raw_data_point_capnp::raw_data_point::Builder>();

                    raw_data_point_builder.set_location(&*self.location);
                    raw_data_point_builder.set_path(&*self.path);
                    raw_data_point_builder.set_component(&*self.component);

                    {
                        let mut date_time_builder = raw_data_point_builder.borrow().init_timestamp();
                        date_time_builder.set_unix_timestamp(self.timestamp.timestamp());
                        date_time_builder.set_nanosecond(self.timestamp.nanosecond());
                    }

                    {
                        let mut value_builder = raw_data_point_builder.borrow().init_value();
                        match self.value {
                            DataValue::Integer(value) => {
                                value_builder.set_integer(value);
                            },
                            DataValue::Float(value) => {
                                value_builder.set_float(value);
                            },
                            DataValue::Bool(value) => {
                                value_builder.set_boolean(value);
                            },
                            DataValue::Text(ref value) => {
                                value_builder.set_text(&*value);
                            },
                        }
                    }
                }

                let mut data = Vec::new();
                match serialize_packed::write_packed_message_unbuffered(&mut data, &mut message) {
                    Ok(_) => {
                        trace!("Message serialized ({} bytes)", data.len());
                        return Ok(data);
                    },
                    Err(error) => {
                        error!("Failed to serialize message for {:?}: {}", <RawDataPoint as SerDeMessage>::data_type(), error);
                        return Err(SerializationError::from(error));
                    }
                }
            },
            Encoding::Plain => {
                warn!("Plain endocing is not implemented for data types");
                // TODO: from?
                Err(From::from(SerDeErrorKind::EncodingNotImplemented(Encoding::Plain)))
            }
        }
    }

    // TODO: use 'type' alias
    fn data_type() -> DataType {
        DataType::RawDataPoint
    }

    fn from_bytes(bytes: &Vec<u8>, encoding: Encoding) -> Result<Self, DeserializationError<Self>> {
        match encoding {
            Encoding::Capnp => {
                let reader = try!(serialize_packed::new_reader_unbuffered(ArrayInputStream::new(bytes), ReaderOptions::new()));
                let raw_data_point = try!(reader.get_root::<::raw_data_point_capnp::raw_data_point::Reader>());

                Ok(
                    RawDataPoint {
                        location: match raw_data_point.get_location() {
                            Ok(value) => value.to_string(),
                            Err(error) => {
                                error!("Failed to read message for {:?}: {}", <RawDataPoint as SerDeMessage>::data_type(), error);
                                return Err(DeserializationError::from(error))
                            }
                        },
                        path: match raw_data_point.get_path() {
                            Ok(value) => value.to_string(),
                            Err(error) => {
                                error!("Failed to read message for {:?}: {}", <RawDataPoint as SerDeMessage>::data_type(), error);
                                return Err(DeserializationError::from(error))
                            }
                        },
                        component: "iowait".to_string(),
                        timestamp: UTC::now(),
                        value: DataValue::Float(0.2)
                    }
                )
            },
            Encoding::Plain => {
                warn!("Plain endocing is not implemented for data types");
                Err(From::from(SerDeErrorKind::EncodingNotImplemented(Encoding::Plain)))
            }
        }
    }
}


#[cfg(test)]
mod test {
    pub use super::*;
    pub use capnp::{MessageBuilder, MallocMessageBuilder};

    #[allow(dead_code)]
    pub mod raw_data_point_capnp {
        include!("./schema/raw_data_point_capnp.rs");
    }

}

