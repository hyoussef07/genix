fn main() {
    // A tiny example showing how to call the library directly.
    let results = genix_lib::generate::generate_many("random", 24, 3, None, false, None).unwrap();
    for r in results {
        println!("{}", r);
    }
}
