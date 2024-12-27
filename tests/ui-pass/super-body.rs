#[spati::spati]
struct Cat {
    age: u8,
}

#[spati::spati]
impl Cat {
    #[super]
    fn new(age: u8) -> Self {
        let age = age * 2;
        Self { age }
    }

    fn age(&self) -> &u8 {
        &self.age
    }
}

fn main() {
    let mut cat = unsafe { Cat::spati_uninit_new() };
    cat.spati_init_new(12);
    assert_eq!(cat.age(), &24);
}
