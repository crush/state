pub trait Backend: serde::de::DeserializeOwned + Sized {
    fn record_state<State>(&self, state: &State) -> Result<(), PersistErr>
        where State: serde::Serialize;
}

pub enum PersistErr {
    PermissionError,
}

#[derive(Deserialize)]
pub struct File {
    pub filename: String,
}

impl Backend for File {
    fn record_state<State>(&self, state: &State) -> Result<(), PersistErr>
        where State: serde::Serialize
    {
        Ok(())
    }
}
