use nanoid::nanoid;

pub trait Entity {
    fn get_id(&self) -> &Option<String>;
    fn set_id(&mut self) -> ();
    fn create_id(&self) -> String {
        nanoid!()
    }
}