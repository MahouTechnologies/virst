use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

use arc_swap::ArcSwapOption;
use inox2d::model::Model;

use crate::tracker::ParamBindings;

#[derive(Default, Debug)]
pub struct DisplayedModel {
    displayed: ArcSwapOption<Model>,
    generation: AtomicU32,
    pub bindings: Mutex<ParamBindings>,
}

impl DisplayedModel {
    pub fn current_model(&self) -> (Option<Arc<Model>>, u32) {
        // Read the generation number first so we can read the displayed
        // This ensures we will never say the generation number changing
        // without `displayed` also changing.
        let generation = self.generation.load(Ordering::Acquire);
        let displayed = self.displayed.load_full();

        (displayed, generation)
    }

    pub fn swap_model(&self, model: Option<Model>) {
        self.displayed.store(model.map(|x| Arc::new(x)));
        self.generation.fetch_add(1, Ordering::Release);
    }
}
