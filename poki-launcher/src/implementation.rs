use super::interface::*;

#[derive(Default, Clone)]
struct AppsModelItem {}

pub struct AppsModel {
    emit: AppsModelEmitter,
    model: AppsModelList,
    list: Vec<AppsModelItem>,
}

impl AppsModelTrait for AppsModel {
    fn new(emit: AppsModelEmitter, model: AppsModelList) -> AppsModel {
        AppsModel {
            emit,
            model,
            list: Vec::new(),
        }
    }
    fn emit(&mut self) -> &mut AppsModelEmitter {
        &mut self.emit
    }
    fn row_count(&self) -> usize {
        self.list.len()
    }
}
