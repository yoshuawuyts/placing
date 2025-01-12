use placing::placing;

#[placing]
struct Bed {
    #[placing]
    cat: Cat,
}

#[placing]
impl Bed {
    #[placing]
    fn new() -> Self {
        Self {
            #[placing]
            cat: Cat::new(12),
        }
    }

    fn cat(&self) -> &Cat {
        &self.cat
    }
}

#[placing]
struct Cat {
    age: u8,
}

#[placing]
impl Cat {
    #[placing]
    fn new(age: u8) -> Self {
        Self { age }
    }
}

fn main() {
    let mut bed = unsafe { Bed::new_uninit() };
    unsafe { bed.new_init() };
    assert_eq!(bed.cat().age(), &12);
}
