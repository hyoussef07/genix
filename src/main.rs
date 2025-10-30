/// Binary entrypoint for the `genix` executable.
///
/// Keeps the binary thin — all business logic lives in the `genix_lib` crate so
/// unit tests can import library functions directly.
fn main() {
    genix_lib::run();
}
