use crate::commands::status::{State, Obj};
use crate::commands::push::new::New;
use crate::commands::push::deleted::Deleted;

#[derive(Debug)]
pub enum PushState {
    Done,
    Valid,
    Conflict,
    Error,
} 

pub trait PushChange {
    fn can_push(&self) -> PushState;
    fn push(&self);
}

pub struct PushFactory;

impl PushFactory {
    pub fn new(&self, obj: Obj) -> Box<dyn PushChange> {
        match obj.state {
            State::New => Box::new(New { obj: obj.clone() }),
            State::Renamed => todo!(),
            State::Modified => todo!(),
            State::Deleted => Box::new(Deleted { obj: obj.clone() }),
            State::Default => todo!(),
        }
    }
}


