// use core::mem::MaybeUninit;

// struct Cat(MaybeUninit<InnerCat>);
// struct InnerCat {
//     age: u8,
// }

// impl Cat {
//     unsafe fn new_uninit() -> Box<MaybeUninit<Cat>> {
//         Box::new(MaybeUninit::uninit())
//     }
//     fn new_init2(&mut self, age: u8) {
//         let this = self.0.as_mut_ptr();
//         unsafe { (&raw mut (*this).age).write(age) };
//     }
//     fn new_init(slot: &mut Box<MaybeUninit<Cat>>, age: u8) {
//         let this = (*slot).as_mut_ptr();
//         let this = unsafe { &mut (*this).0 }.as_mut_ptr();
//         unsafe { (&raw mut (*this).age).write(age) };
//     }
// }

fn main() {}
