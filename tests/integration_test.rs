#[test]
fn integration_generate_and_entropy() {
    // Generate some items and verify sanity and entropy estimation
    let res =
        genix_lib::generate::generate_many("random", 32, 2, None, false, None).expect("generate");
    assert_eq!(res.len(), 2);
    let e = genix_lib::entropy::estimate_entropy_for_str(&res[0], "random").expect("entropy");
    assert!(e > 0.0);
}
