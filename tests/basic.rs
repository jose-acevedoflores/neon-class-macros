use neon_macros::*;

#[test]
fn what() {
    #[my_attribute]
    struct Wut;

    let _w = Wut;
}
