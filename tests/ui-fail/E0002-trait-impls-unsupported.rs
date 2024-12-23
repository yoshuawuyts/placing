struct Cat {
    age: u8,
}

trait Meow {}

#[spati::spati]
impl Meow for Cat {}

fn main() {}
