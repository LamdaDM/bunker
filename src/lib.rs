pub mod server;
pub mod exception;
#[allow(deprecated)]
pub mod registerable;

mod internal;
mod cfg;

#[cfg(test)]
mod tests {
    use crate::{registerable::ParseOptions, internal::Threadpool};

    #[test]
    fn parse_options_separators_check() {
        assert!(ParseOptions::separator(vec![]).is_separators())
    }

    #[test]
    fn worker_count() {
        let threadpool = Threadpool::new(3);
        let mock: Vec<usize> = vec![0, 1, 2];

        assert_eq!(mock, threadpool.get_all_workerid());
    }

    #[test]
    fn threadpool_size() {
        let threadpool = Threadpool::new(2);

        assert_eq!(threadpool.get_size(), 2);
    }
}