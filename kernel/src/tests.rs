pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Total test Job {}", tests.len());
    for f in tests {
        f();
    }
}