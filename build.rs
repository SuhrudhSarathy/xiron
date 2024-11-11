fn main() {
    prost_build::compile_protos(
        &[
            "src/protos/twist.proto",
            "src/protos/pose.proto",
            "src/protos/laser_scan.proto",
            "src/protos/reset.proto",
        ],
        &["src/"],
    )
    .unwrap();
}
