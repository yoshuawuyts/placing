#[placing::placing]
struct Cat {
    age: u8,
}

#[placing::placing]
impl Cat {
    #[placing]
    fn new(age: u8) -> Self {
        Self { age }
    }

    fn age(&self) -> &u8 {
        &self.age
    }
}

fn main() {
    let mut cat = unsafe { Cat::new_uninit() };
    unsafe { cat.new_init(12) };
    assert_eq!(cat.age(), &12);
}
