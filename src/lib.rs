// extern crate http;
extern crate hyper;
extern crate futures;

pub mod router;
pub mod tree;
pub mod path;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
