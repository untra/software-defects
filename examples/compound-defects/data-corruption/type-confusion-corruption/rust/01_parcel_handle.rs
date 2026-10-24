const INITIAL_CODE: u32 = 7;
const INITIAL_CREDIT: u32 = 100;
const IMPORTED_CODE: u64 = (9_u64 << u32::BITS) | 8;

#[repr(C)]
struct Parcel { code: u32, credit: u32 }

fn store_code(code: *mut u32, value: u64) {
    let destination = code.cast::<u64>();
    unsafe { destination.write_unaligned(value); }
}

fn main() {
    let mut parcel = Parcel { code: INITIAL_CODE, credit: INITIAL_CREDIT };
    store_code(&mut parcel.code, IMPORTED_CODE);
    println!("credit {}", parcel.credit);
}
