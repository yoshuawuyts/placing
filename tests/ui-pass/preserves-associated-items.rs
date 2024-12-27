#[spati::spati]
struct Cat {}

#[spati::spati]
impl Cat {
    const NAME: &str = "chashu";
}

fn main() {
    assert_eq!(Cat::NAME, "chashu");
}
