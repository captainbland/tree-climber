extern crate climb;

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn parse() {
        let testData = "
           .
           ├── data
           │   ├── comments.rs
           │   ├── listing.rs
           │   ├── mod.rs
           │   ├── post.rs
           │   ├── sub.rs
           │   ├── thing.rs
           │   └── user.rs
           ├── errors.rs
           ├── mod.rs
           ├── net
           │   ├── auth.rs
           │   └── mod.rs
           └── tests.rs
           ";
    }

    #[test]
    fn test_add() {
        process_cli_tree()
    }

    #[test]
    fn test_bad_add() {
        // This assert would fire and tests will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(bad_add(1, 2), 3);
    }
}
