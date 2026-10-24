const SAMPLES: [u32; 0] = [];
fn mean(samples: &[u32]) -> u32 {
    let mut total = 0;
    for sample in samples { total += sample; }
    total / samples.len() as u32
}
fn main() { println!("mean {}", mean(&SAMPLES)); }
