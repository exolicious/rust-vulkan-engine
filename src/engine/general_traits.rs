pub trait Entity {
    fn get_id(&self) -> &String;
    fn update(&mut self) -> ();
}