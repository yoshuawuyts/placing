#[placing::placing]
struct Cat {
    age: u8,
}

#[placing::placing]
impl Cat {
    #[placing]
    fn new(age: u8) -> Box<Self> {
        Box::new(Self { age })
    }

    fn age(&self) -> &u8 {
        &self.age
    }
}

fn main() {
    let mut cat = unsafe { Cat::placing_uninit_new() };
    unsafe { cat.placing_init_new(12) };
    assert_eq!(cat.age(), &12);
}
