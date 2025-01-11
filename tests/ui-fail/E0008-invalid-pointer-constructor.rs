#[spati::spati]
struct Cat {
    age: u8,
}

#[spati::spati]
impl Cat {
    #[placing]
    fn new(age: u8) -> Box<Self> {
        Box::new()
    }
}

fn main() {}
