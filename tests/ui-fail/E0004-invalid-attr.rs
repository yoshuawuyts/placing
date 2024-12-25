#[spati::spati]
struct Cat;

#[spati::spati]
impl Cat {
    #[super(invalid)]
    fn list(&self) -> Self {
        todo!()
    }

    #[super = "invalid"]
    fn path(&self) -> Self {
        todo!()
    }
}

fn main() {}
