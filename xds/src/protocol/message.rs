/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use byteorder::{BigEndian, ByteOrder};
use enum_primitive_derive::Primitive;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use num_traits::{FromPrimitive, ToPrimitive};
use strum_macros::{Display, EnumIter, EnumString};

use std::{
    cell::RefCell,
    collections::hash_map::HashMap,
    io::{Read, Write},
};

use super::error::*;


const MAGIC_NUMBER: u8 = 0x08;
pub const SERVICE_ERROR: &str = "__rpcx_error__";

#[derive(Debug, Copy, Clone, Display, PartialEq, EnumIter, EnumString, Primitive)]
pub enum MessageType {
    Request = 0,
    Response = 1,
}

#[derive(Debug, Copy, Clone, Display, PartialEq, EnumIter, EnumString, Primitive)]
pub enum MessageStatusType {
    Normal = 0,
    Error = 1,
}

#[derive(Debug, Copy, Clone, Display, PartialEq, EnumIter, EnumString, Primitive)]
pub enum CompressType {
    CompressNone = 0,
    Gzip = 1,
}

#[derive(Debug, Copy, Clone, Display, PartialEq, EnumIter, EnumString, Primitive)]
pub enum SerializeType {
    SerializeNone = 0,
    JSON = 1,
    Protobuf = 2,
    MsgPack = 3,
    Thrift = 4,
}

/// define the rpcx message interface.
pub trait RpcxMessage {
    fn check_magic_number(&self) -> bool;
    fn get_version(&self) -> u8;
    fn set_version(&mut self, v: u8);
    fn get_message_type(&self) -> Option<MessageType>;
    fn set_message_type(&mut self, mt: MessageType);
    fn is_heartbeat(&self) -> bool;
    fn set_heartbeat(&mut self, b: bool);
    fn is_oneway(&self) -> bool;
    fn set_oneway(&mut self, b: bool);
    fn get_compress_type(&self) -> Option<CompressType>;
    fn set_compress_type(&mut self, ct: CompressType);
    fn get_message_status_type(&self) -> Option<MessageStatusType>;
    fn set_message_status_type(&mut self, mst: MessageStatusType);
    fn get_serialize_type(&self) -> Option<SerializeType>;
    fn set_serialize_type(&mut self, st: SerializeType);
    fn get_seq(&self) -> u64;
    fn set_seq(&mut self, seq: u64);
    fn decode<R: ?Sized>(&mut self, r: &mut R) -> Result<()>
    where
        R: Read;
    fn encode(&self) -> Vec<u8>;

    fn get_error(&self) -> Option<String>;
}

pub type Metadata = HashMap<String, String>;

/// a commmon struct for request and response.
#[derive(Debug, Default)]
pub struct Message {
    pub header: [u8; 12],
    pub service_path: String,
    pub service_method: String,
    pub metadata: RefCell<Metadata>,
    pub payload: Vec<u8>,
}
impl Message {
    /// Creates a new `Message`
    pub fn new() -> Self {
        let mut msg: Message = Default::default();
        msg.header = [0u8; 12];
        msg.header[0] = MAGIC_NUMBER;
        msg.metadata = RefCell::new(HashMap::new());
        msg
    }

    pub fn get_reply(&self) -> Result<Self> {
        let mut reply = Message::new();
        reply.set_version(self.get_version());
        reply.set_compress_type(self.get_compress_type().unwrap());
        reply.set_message_status_type(MessageStatusType::Normal);
        reply.set_message_type(MessageType::Response);
        reply.set_serialize_type(self.get_serialize_type().unwrap());
        reply.set_seq(self.get_seq());
        reply.service_path = self.service_path.clone();
        reply.service_method = self.service_method.clone();

        Ok(reply)
    }
}

impl RpcxMessage for Message {
    fn check_magic_number(&self) -> bool {
        self.header[0] == MAGIC_NUMBER
    }

    fn get_version(&self) -> u8 {
        self.header[1]
    }
    fn set_version(&mut self, v: u8) {
        self.header[1] = v;
    }

    fn get_message_type(&self) -> Option<MessageType> {
        MessageType::from_u8((self.header[2] & 0x80) >> 7 as u8)
    }
    fn set_message_type(&mut self, mt: MessageType) {
        self.header[2] |= mt.to_u8().unwrap() << 7;
    }
    fn is_heartbeat(&self) -> bool {
        self.header[2] & 0x40 == 0x40
    }
    fn set_heartbeat(&mut self, b: bool) {
        if b {
            self.header[2] |= 0x40;
        } else {
            self.header[2] &= !0x40;
        }
    }
    fn is_oneway(&self) -> bool {
        self.header[2] & 0x20 == 0x20
    }
    fn set_oneway(&mut self, b: bool) {
        if b {
            self.header[2] |= 0x20;
        } else {
            self.header[2] &= !0x20;
        }
    }
    fn get_compress_type(&self) -> Option<CompressType> {
        CompressType::from_u8((self.header[2] & 0x1C) >> 2)
    }
    fn set_compress_type(&mut self, ct: CompressType) {
        self.header[2] = (self.header[2] & !0x1C) | (ct.to_u8().unwrap() << 2 & 0x1C);
    }
    fn get_message_status_type(&self) -> Option<MessageStatusType> {
        MessageStatusType::from_u8(self.header[2] & 0x03)
    }
    fn set_message_status_type(&mut self, mst: MessageStatusType) {
        self.header[2] = (self.header[2] & !0x03) | (mst.to_u8().unwrap() & 0x03);
    }
    fn get_serialize_type(&self) -> Option<SerializeType> {
        SerializeType::from_u8((self.header[3] & 0xF0) >> 4)
    }
    fn set_serialize_type(&mut self, st: SerializeType) {
        self.header[3] = (self.header[3] & !0xF0) | (st.to_u8().unwrap() << 4)
    }
    fn get_seq(&self) -> u64 {
        u64_from_slice(&(self.header[4..]))
    }
    fn set_seq(&mut self, seq: u64) {
        u64_to_slice(seq, &mut self.header[4..]);
    }

    fn decode<R: ?Sized>(&mut self, r: &mut R) -> Result<()>
    where
        R: Read,
    {
        r.read_exact(&mut self.header)?;

        let mut buf = [0u8; 4];
        r.read(&mut buf[..]).map(|_| {})?;
        let len = BigEndian::read_u32(&buf); //length of all expect header
        let mut buf = vec![0u8; len as usize];
        r.read(&mut buf[..]).map(|_| ())?;

        let mut start = 0;
        // read service_path
        let len = read_len(&buf[start..(start + 4)]) as usize;
        let service_path = read_str(&buf[(start + 4)..(start + 4 + len)])?;
        self.service_path = service_path;
        start = start + 4 + len;
        // read service_method
        let len = read_len(&buf[start..(start + 4)]) as usize;
        let service_method = read_str(&buf[(start + 4)..(start + 4 + len)])?;
        self.service_method = service_method;

        start = start + 4 + len;
        //metadata
        let len = read_len(&buf[start..(start + 4)]) as usize;
        let metadata_bytes = &buf[(start + 4)..(start + 4 + len)];
        let mut meta_start = 0;
        while meta_start < len {
            let sl = read_len(&metadata_bytes[meta_start..(meta_start + 4)]) as usize;
            let key = read_str(&metadata_bytes[(meta_start + 4)..(meta_start + 4 + sl)])?;
            meta_start = meta_start + 4 + sl;
            if meta_start < len {
                let value_len = read_len(&metadata_bytes[meta_start..(meta_start + 4)]) as usize;
                let value =
                    read_str(&metadata_bytes[(meta_start + 4)..(meta_start + 4 + value_len)])?;
                self.metadata.borrow_mut().insert(key, value);
                meta_start = meta_start + 4 + value_len;
            } else {
                self.metadata.borrow_mut().insert(key, String::new());
                break;
            }
        }
        start = start + 4 + len;
        // payload
        let len = read_len(&buf[start..start + 4]) as usize;
        let payload = &buf[start + 4..];
        if len != payload.len() {
            return Err(Error::from("invalid payload length"));
        }

        let mut vp = Vec::with_capacity(payload.len());
        match self.get_compress_type().unwrap() {
            CompressType::Gzip => {
                let mut deflater = GzDecoder::new(payload);
                deflater.read_to_end(&mut vp)?;
            }
            CompressType::CompressNone => {
                vp.extend_from_slice(&payload);
            }
        }
        self.payload = vp;

        Ok(())
    }

    fn encode(&self) -> Vec<u8> {
        // encode all except header
        let mut buf = Vec::<u8>::with_capacity(20);
        buf.extend_from_slice(&self.header);

        // push fake length
        let len_bytes = write_len(0);
        buf.extend_from_slice(&len_bytes);

        // service_path
        let len = self.service_path.len();
        let len_bytes = write_len(len as u32);
        buf.extend_from_slice(&len_bytes);
        buf.extend_from_slice(self.service_path.as_bytes());

        // service_method
        let len = self.service_method.len();
        let len_bytes = write_len(len as u32);
        buf.extend_from_slice(&len_bytes);
        buf.extend_from_slice(self.service_method.as_bytes());

        // metadata
        let mut metadata_bytes = Vec::<u8>::new();
        let metadata = self.metadata.borrow_mut();
        for meta in metadata.iter() {
            let key = meta.0;
            let len_bytes = write_len(key.len() as u32);
            metadata_bytes.extend_from_slice(&len_bytes);
            metadata_bytes.extend_from_slice(key.as_bytes());

            let value = meta.1;
            let len_bytes = write_len(value.len() as u32);
            metadata_bytes.extend_from_slice(&len_bytes);
            metadata_bytes.extend_from_slice(value.as_bytes());
        }
        let len = metadata_bytes.len();
        let len_bytes = write_len(len as u32);
        buf.extend_from_slice(&len_bytes);
        buf.append(&mut metadata_bytes);

        // data
        // check compress

        match self.get_compress_type().unwrap() {
            CompressType::Gzip => {
                let mut e = GzEncoder::new(Vec::new(), Compression::fast());
                let _ = e.write_all(&self.payload[..]);
                let compressed_payload = e.finish().unwrap();
                let len = compressed_payload.len();
                let len_bytes = write_len(len as u32);
                buf.extend_from_slice(&len_bytes);
                buf.extend_from_slice(&compressed_payload);
            }
            _ => {
                let len = self.payload.len();
                let len_bytes = write_len(len as u32);
                buf.extend_from_slice(&len_bytes);
                buf.extend_from_slice(&self.payload);
            }
        }

        // set the real length
        let len = buf.len() - 12 - 4;
        let len_bytes = write_len(len as u32);
        buf[12] = len_bytes[0];
        buf[13] = len_bytes[1];
        buf[14] = len_bytes[2];
        buf[15] = len_bytes[3];

        buf
    }

    fn get_error(&self) -> Option<String> {
        match self.get_message_status_type() {
            Some(MessageStatusType::Error) => {
                let metadata = &self.metadata;
                let metadata2 = metadata.borrow();
                let err_msg = metadata2.get(&SERVICE_ERROR.to_owned())?;
                Some(String::from(err_msg))
            }
            _ => None,
        }
    }
}

fn read_len(buf: &[u8]) -> u32 {
    BigEndian::read_u32(&buf[..4])
}

fn write_len(len: u32) -> [u8; 4] {
    let mut buf = [0u8; 4];
    BigEndian::write_u32(&mut buf, len);
    buf
}

fn read_str(buf: &[u8]) -> Result<String> {
    let s = std::str::from_utf8(&buf).unwrap();
    let str: String = std::string::String::from(s);
    Ok(str)
}

fn u64_from_slice(b: &[u8]) -> u64 {
    BigEndian::read_u64(b)
}

fn u64_to_slice(v: u64, b: &mut [u8]) {
    BigEndian::write_u64(b, v);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_header() {
        let msg_data: Vec<u8> = vec![
            8, 0, 0, 16, 0, 0, 0, 0, 73, 150, 2, 210, 0, 0, 0, 98, 0, 0, 0, 5, 65, 114, 105, 116,
            104, 0, 0, 0, 3, 65, 100, 100, 0, 0, 0, 48, 0, 0, 0, 4, 95, 95, 73, 68, 0, 0, 0, 36,
            54, 98, 97, 55, 98, 56, 49, 48, 45, 57, 100, 97, 100, 45, 49, 49, 100, 49, 45, 56, 48,
            98, 52, 45, 48, 48, 99, 48, 52, 102, 100, 52, 51, 48, 99, 57, 0, 0, 0, 26, 123, 10, 9,
            9, 34, 65, 34, 58, 32, 49, 44, 10, 9, 9, 34, 66, 34, 58, 32, 50, 44, 10, 9, 125, 10, 9,
        ];

        let mut msg = Message::new();
        (&mut msg.header).copy_from_slice(&msg_data[..12]);

        assert_eq!(true, msg.check_magic_number());
        assert_eq!(0, msg.get_version());
        assert_eq!(MessageType::Request, msg.get_message_type().unwrap());
        assert_eq!(false, msg.is_heartbeat());
        assert_eq!(false, msg.is_oneway());
        assert_eq!(CompressType::CompressNone, msg.get_compress_type().unwrap());
        assert_eq!(
            MessageStatusType::Normal,
            msg.get_message_status_type().unwrap()
        );
        assert_eq!(SerializeType::JSON, msg.get_serialize_type().unwrap());
        assert_eq!(1234567890, msg.get_seq());
    }

    #[test]
    fn set_header() {
        let msg_data: Vec<u8> = vec![
            8, 0, 0, 16, 0, 0, 0, 0, 73, 150, 2, 210, 0, 0, 0, 98, 0, 0, 0, 5, 65, 114, 105, 116,
            104, 0, 0, 0, 3, 65, 100, 100, 0, 0, 0, 48, 0, 0, 0, 4, 95, 95, 73, 68, 0, 0, 0, 36,
            54, 98, 97, 55, 98, 56, 49, 48, 45, 57, 100, 97, 100, 45, 49, 49, 100, 49, 45, 56, 48,
            98, 52, 45, 48, 48, 99, 48, 52, 102, 100, 52, 51, 48, 99, 57, 0, 0, 0, 26, 123, 10, 9,
            9, 34, 65, 34, 58, 32, 49, 44, 10, 9, 9, 34, 66, 34, 58, 32, 50, 44, 10, 9, 125, 10, 9,
        ];

        let mut msg = Message::new();
        msg.header.copy_from_slice(&msg_data[..12]);

        msg.set_version(0);
        msg.set_message_type(MessageType::Response);
        msg.set_heartbeat(true);
        msg.set_oneway(true);
        msg.set_compress_type(CompressType::Gzip);
        msg.set_serialize_type(SerializeType::MsgPack);
        msg.set_message_status_type(MessageStatusType::Normal);
        msg.set_seq(1000000);

        assert_eq!(true, msg.check_magic_number());
        assert_eq!(0, msg.get_version());
        assert_eq!(MessageType::Response, msg.get_message_type().unwrap());
        assert_eq!(true, msg.is_heartbeat());
        assert_eq!(true, msg.is_oneway());
        assert_eq!(CompressType::Gzip, msg.get_compress_type().unwrap());
        assert_eq!(
            MessageStatusType::Normal,
            msg.get_message_status_type().unwrap()
        );
        assert_eq!(SerializeType::MsgPack, msg.get_serialize_type().unwrap());
        assert_eq!(1000000, msg.get_seq());
    }

    #[test]
    fn decode() {
        let msg_data: [u8; 114] = [
            8, 0, 0, 16, 0, 0, 0, 0, 73, 150, 2, 210, 0, 0, 0, 98, 0, 0, 0, 5, 65, 114, 105, 116,
            104, 0, 0, 0, 3, 65, 100, 100, 0, 0, 0, 48, 0, 0, 0, 4, 95, 95, 73, 68, 0, 0, 0, 36,
            54, 98, 97, 55, 98, 56, 49, 48, 45, 57, 100, 97, 100, 45, 49, 49, 100, 49, 45, 56, 48,
            98, 52, 45, 48, 48, 99, 48, 52, 102, 100, 52, 51, 48, 99, 57, 0, 0, 0, 26, 123, 10, 9,
            9, 34, 65, 34, 58, 32, 49, 44, 10, 9, 9, 34, 66, 34, 58, 32, 50, 44, 10, 9, 125, 10, 9,
        ];

        let mut msg = Message::new();

        let mut data = &msg_data[..] as &[u8];
        match msg.decode(&mut data) {
            Err(err) => println!("failed to parse: {}", err),
            Ok(()) => {}
        }

        assert_eq!("Arith", msg.service_path);
        assert_eq!("Add", msg.service_method);

        assert_eq!(
            "6ba7b810-9dad-11d1-80b4-00c04fd430c9",
            msg.metadata.borrow().get("__ID").unwrap()
        );

        assert_eq!(
            "{\n\t\t\"A\": 1,\n\t\t\"B\": 2,\n\t}\n\t",
            std::str::from_utf8(&msg.payload).unwrap()
        );
    }

    #[test]
    fn encode() {
        let msg_data: [u8; 114] = [
            8, 0, 0, 16, 0, 0, 0, 0, 73, 150, 2, 210, 0, 0, 0, 98, 0, 0, 0, 5, 65, 114, 105, 116,
            104, 0, 0, 0, 3, 65, 100, 100, 0, 0, 0, 48, 0, 0, 0, 4, 95, 95, 73, 68, 0, 0, 0, 36,
            54, 98, 97, 55, 98, 56, 49, 48, 45, 57, 100, 97, 100, 45, 49, 49, 100, 49, 45, 56, 48,
            98, 52, 45, 48, 48, 99, 48, 52, 102, 100, 52, 51, 48, 99, 57, 0, 0, 0, 26, 123, 10, 9,
            9, 34, 65, 34, 58, 32, 49, 44, 10, 9, 9, 34, 66, 34, 58, 32, 50, 44, 10, 9, 125, 10, 9,
        ];

        let mut msg = Message::new();

        let mut data = &msg_data[..] as &[u8];
        match msg.decode(&mut data) {
            Err(err) => println!("failed to parse: {}", err),
            Ok(()) => {}
        }

        let encoded_bytes = msg.encode();

        assert_eq!(&msg_data[..], &encoded_bytes[..]);
    }
}
