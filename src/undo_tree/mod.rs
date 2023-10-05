use crate::table::Data;

#[derive(Clone)]
pub struct UndoTree {
    table_rows: Box<Vec<Vec<Data>>>,
    previous: Option<Box<UndoTree>>,
    next: Option<Box<UndoTree>>,
    id: usize,
}

impl UndoTree {
    pub fn new(initial_state: Box<Vec<Vec<Data>>>, id: usize) -> UndoTree {
        UndoTree {
            table_rows: initial_state,
            previous: None,
            next: None,
            id,
        }
    }

    pub fn save(&mut self, state: Box<Vec<Vec<Data>>>) -> UndoTree {
        let mut tree = UndoTree::new(state, self.id + 1);
        self.next = Some(Box::new(tree.clone()));
        tree.previous = Some(Box::new(self.clone()));
        return tree;
    }

    pub fn undo(&self) -> Option<Box<UndoTree>> {
        return self.previous.clone();
    }

    pub fn redo(&self) -> Option<Box<UndoTree>> {
        return self.next.clone();
    }

    pub fn get_state(&self) -> Box<Vec<Vec<Data>>> {
        return self.table_rows.clone();
    }

    pub fn get_id(&self) -> usize{
        return self.id;
    }
}
