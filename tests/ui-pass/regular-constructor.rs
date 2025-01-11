#[placing::placing]
struct Cat {
    age: u8,
}

#[placing::placing]
impl Cat {
    fn new(age: u8) -> Self {
        Self { age }
    }

    fn age(&self) -> &u8 {
        &self.age
    }
}

fn main() {
    let cat = Cat::new(12);
    assert_eq!(cat.age(), &12);
}
