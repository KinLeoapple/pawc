use std::collections::HashMap;
use crate::semantic::scope::PawType;

pub struct InterfaceSig {
    pub method_sigs: HashMap<String, (Vec<PawType>, PawType, bool)>,
}