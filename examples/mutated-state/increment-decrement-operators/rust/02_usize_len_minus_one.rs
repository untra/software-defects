const PAGE_SIZE: usize = 10;

fn last_page(items: &[u32], page_size: usize) -> usize {
    (items.len() - 1) / page_size
}

fn main() {
    let results: Vec<u32> = Vec::new();
    println!("last page: {}", last_page(&results, PAGE_SIZE));
}
