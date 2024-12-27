#[spati::spati]
struct Cat<K, J, const N: usize>
where
    J: Send,
{
    k: K,
    j: J,
    foo: [u8; N],
}

#[spati::spati]
impl<K, J, const N: usize> Cat<K, J, N> where J: Send {}

fn main() {}
