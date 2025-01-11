use placing::placing;

#[placing]
struct Bed {
    #[placing]
    cat: Cat,
}

#[placing]
struct Cat {
    age: u8,
}

fn main() {}
