#[placing::placing]
struct Cat {
    age: u8,
}

#[placing::placing]
impl Cat {
    #[placing]
    fn new(age: u8) -> Box<Self> {
        Box::new()
    }
}

fn main() {}
