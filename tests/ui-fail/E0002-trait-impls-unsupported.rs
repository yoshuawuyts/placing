struct Cat {
    age: u8,
}

trait Meow {}

#[placing::placing]
impl Meow for Cat {}

fn main() {}
