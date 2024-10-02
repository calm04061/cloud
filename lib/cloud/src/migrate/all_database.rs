use crate::migrate::Migrate;

struct AllDatabase{

}
impl Migrate for AllDatabase {
    fn version(&self) -> String {
        todo!()
    }

    fn do_migrate(&mut self) {
        todo!()
    }
}