#[spati::spati]
struct Cat<K, const N: usize> {
    k: K,
    foo: [u8; N],
}

#[spati::spati]
impl<K, const N: usize> Cat<K, N> {}

fn main() {}
