#[placing::placing]
struct Cat {}

#[placing::placing]
impl Cat {
    const NAME: &str = "chashu";
}

fn main() {
    assert_eq!(Cat::NAME, "chashu");
}
