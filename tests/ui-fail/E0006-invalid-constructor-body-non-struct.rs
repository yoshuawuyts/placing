#[spati::spati]
struct Cat;

#[spati::spati]
impl Cat {
    #[super]
    fn new() -> Self {
        {
            Self {}
        }
    }
}

fn main() {}
