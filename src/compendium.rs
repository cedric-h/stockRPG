//use crate::prelude::*;
use imgui::ImString;

pub struct Compendium {
    pub entity_query: ImString,
    pub wip_type_name: ImString,
    pub component_to_add_index: i32,
    pub editing_assemblage: Option<String>,
    pub place_assemblage: Option<String>,
    pub place_me_entity: Option<specs::Entity>,
    pub chosen_entity: Option<specs::Entity>,
}

impl Compendium {
    pub fn new() -> Self {
        Self {
            entity_query: ImString::with_capacity(25),
            wip_type_name: ImString::with_capacity(25),
            component_to_add_index: 0,
            editing_assemblage: None,
            place_assemblage: None,
            place_me_entity: None,
            chosen_entity: None,
        }
    }
}
