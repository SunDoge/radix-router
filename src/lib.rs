// extern crate http;
extern crate hyper;
extern crate futures;
extern crate tokio_fs;
extern crate tokio_io;

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
