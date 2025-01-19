pub mod mysql;

pub trait Cleaner {
    fn clean(&self);
    fn get_size_of_database(&self) -> u64;
}
