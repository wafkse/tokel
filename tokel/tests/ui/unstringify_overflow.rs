tokel::stream! {
    const FLAT: u8 = [< "256" >]:unstringify;
    const NESTED: [u8; 1] = [< "[256]" >]:unstringify;
}

fn main() {}
