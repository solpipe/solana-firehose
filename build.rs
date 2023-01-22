fn main() {
    tonic_build::configure()
        .build_server(true)
        .compile(&[
            "./proto/basic.proto",
            "./proto/pipe.proto",
        ], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

}

