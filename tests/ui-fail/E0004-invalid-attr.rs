#[spati::spati]
struct Cat;

#[spati::spati]
impl Cat {
    #[placing(invalid)]
    fn list(&self) -> Self {
        todo!()
    }

    #[placing = "invalid"]
    fn path(&self) -> Self {
        todo!()
    }
}

fn main() {}
