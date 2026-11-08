const INITIAL_COUNT: u8 = u8::MAX;
const EXTRA_PARCELS: u8 = 2;
const LABEL: u8 = 7;
fn main() {
    let mut count = INITIAL_COUNT;
    count = count.wrapping_add(EXTRA_PARCELS);
    let mut labels = vec![0_u8; usize::from(count)];
    for index in 0..usize::from(EXTRA_PARCELS) {
        labels[index] = LABEL;
    }
    println!("labels {:?}", labels);
}
