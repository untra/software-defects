const NAME_BYTES: usize = 4;
const INPUT: &[u8] = b"pearsXYZ";
#[repr(C)]
struct Shipment { name: [u8; NAME_BYTES], units: [u8; NAME_BYTES] }
fn main() {
    let mut shipment = Shipment { name: [0; NAME_BYTES], units: *b"0012" };
    let name = shipment.name.as_mut_ptr();
    unsafe { std::ptr::copy_nonoverlapping(INPUT.as_ptr(), name, INPUT.len()); }
    println!("units {}", String::from_utf8_lossy(&shipment.units));
}
