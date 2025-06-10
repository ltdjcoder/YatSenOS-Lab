fn humanized_size(size: u64) -> (f64, &'static str) {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    const THRESHOLD: f64 = 1024.0;
    
    if size == 0 {
        return (0.0, "B");
    }
    
    let mut size_float = size as f64;
    let mut unit_index = 0;
    
    while size_float >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size_float /= THRESHOLD;
        unit_index += 1;
    }
    
    (size_float, UNITS[unit_index])
}

fn main() {
    println!("Hello, world!");
    
    // 测试一些例子
    let test_sizes = vec![0, 1024, 1554056, 1073741824, 1099511627776];
    
    for size in test_sizes {
        let (humanized, unit) = humanized_size(size);
        println!("Size: {:7.4} {}", humanized, unit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_humanized_size() {
        let byte_size = 1554056;
        let (size, unit) = humanized_size(byte_size);
        assert_eq!("Size :  1.4821 MiB", format!("Size : {:7.4} {}", size, unit));
    }
}
