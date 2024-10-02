mod all_database;

trait Migrate {
    fn version(&self) -> String;
    fn do_migrate(&mut self);
}