use crate::prelude::*;
use imgui::ImString;
use specs::{Entities, Entity, WriteStorage};

pub struct Compendium {
    pub entity_query: ImString,
    pub wip_type_name: ImString,
    pub component_to_add_index: i32,
    pub mouselock_chosen_ent: bool,
    pub editing_assemblage: Option<String>,
    pub place_assemblage: Option<String>,
    pub chosen_ent: Option<Entity>,
}

impl Compendium {
    pub fn new() -> Self {
        Self {
            entity_query: ImString::with_capacity(25),
            wip_type_name: ImString::with_capacity(25),
            component_to_add_index: 0,
            mouselock_chosen_ent: false,
            editing_assemblage: None,
            place_assemblage: None,
            chosen_ent: None,
        }
    }

    pub fn choose_ent(
        &mut self,
        ent: Entity,
        ents: &Entities,
        outlines: &mut WriteStorage<BoxOutline>,
    ) {
        let outline = self
            .chosen_ent
            .map(|old_ent| {
                let save_box = outlines.get(old_ent).unwrap().clone();

                // delete whatever they were about to place before if that's a thing
                // or if they weren't about to delete it still remove its outline.
                if self.mouselock_chosen_ent {
                    ents.delete(old_ent).unwrap();
                } else {
                    outlines.remove(old_ent);
                }

                save_box
            })
            .unwrap_or(BoxOutline {
                color: [0.2, 0.5, 0.6],
                fade: [0.015, 0.075, 0.130, 0.205],
            });
        outlines.insert(ent, outline).unwrap();
        self.chosen_ent = Some(ent);
    }

    pub fn get_chosen_ent(&self) -> Option<Entity> {
        self.chosen_ent
    }

    pub fn unchoose_ent(&mut self, outlines: &mut WriteStorage<BoxOutline>) {
        info!("well at least it's called I guess");
        if let Some(ent) = self.chosen_ent {
            info!("I don't get it; I erradicated the mfer!");
            outlines.remove(ent);
        }
        self.chosen_ent = None;
        self.mouselock_chosen_ent = false;
    }
}
