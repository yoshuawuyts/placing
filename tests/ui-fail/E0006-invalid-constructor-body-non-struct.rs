#[placing::placing]
struct Cat;

#[placing::placing]
impl Cat {
    #[placing]
    fn new() -> Self {
        {
            Self {}
        }
    }
}

fn main() {}
