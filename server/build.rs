extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["../protobuf/packets.proto"], &["../protobuf/"]).unwrap();
}
