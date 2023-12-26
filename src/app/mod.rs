use self::model::Models;

mod model;

#[derive(Debug, Default)]
pub struct AppState {
    models: Models,
}
