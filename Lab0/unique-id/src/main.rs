mod unique_id;
use unique_id::UniqueId;

fn main() {
    let id = UniqueId::new();
    println!("Generated unique ID: {:?}", id);
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_unique_id() {
        let id1 = UniqueId::new();
        let id2 = UniqueId::new();
        assert_ne!(id1, id2);
    }
}