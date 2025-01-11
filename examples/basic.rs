#[spati::spati]
struct Cat {
    age: u8,
}

#[spati::spati]
impl Cat {
    #[super]
    fn new(age: u8) -> Box<Self> {
        Box::new(Self { age })
    }

    fn age(&self) -> &u8 {
        &self.age
    }
}

fn main() {
    let mut cat = unsafe { Cat::spati_uninit_new() };
    unsafe { Cat::spati_init_new(&mut cat, 12) };
    assert_eq!(cat.age(), &12);
}
