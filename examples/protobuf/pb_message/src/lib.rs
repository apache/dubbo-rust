
pub mod pb;

#[cfg(test)]
mod tests {
    use crate::pb::Person;
    use crate::pb::greeter;

    use std::io::Cursor;
    use prost::Message;

    pub fn create_hello_request(name: String) -> greeter::HelloRequest {
        let mut hello_request = greeter::HelloRequest::default();
        hello_request.name = name;
        hello_request
    }

    pub fn serialize_greeter(hello: &greeter::HelloRequest) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.reserve(hello.encoded_len());

        hello.encode(&mut buf).unwrap();
        buf
    }


    // pub fn deserialize_greeter(buf: &[u8]) -> Result<greeter::HelloRequest, prost::DecodeError> {
    //     greeter::HelloRequest::decode(&mut Cursor::new(buf))
    // }

    #[test]
    fn it_works() {
        let person = Person::default();
        person.encode_to_vec();
        println!("{person:?}");
    }
}
