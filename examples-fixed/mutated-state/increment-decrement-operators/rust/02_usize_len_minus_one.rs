const PAGE_SIZE: usize = 10;

fn last_page(items: &[u32], page_size: usize) -> Option<usize> {
    items.len().checked_sub(1)?.checked_div(page_size)
}

fn main() {
    let results: Vec<u32> = Vec::new();
    println!("last page: {:?}", last_page(&results, PAGE_SIZE));
}
