// This package does not compile protos directly.
// Each service's build.rs compiles its proto and outputs to this package.
//
// Example for user-service/build.rs:
// ```rust
// tonic_build::configure()
//     .build_server(true)
//     .out_dir("../../../api/src")
//     .compile_protos(&["src/pb/user.proto"], &["src/pb"])?;
// ```
//
// This allows all API types to be centralized in the `api` crate
// while keeping proto definitions close to their service implementations.
fn main() {}
