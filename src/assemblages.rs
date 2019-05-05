use crate::prelude::*;
use custom_component_macro::AssemblageComponent;
use imgui::ImString;
use specs::{world::EntitiesRes, world::LazyBuilder, Entity, LazyUpdate};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

const TYPES_PATH: &str = "./src/data/types.json";
const INSTANCES_PATH: &str = "./src/data/instances.json";

#[allow(dead_code)]
pub struct Assemblager {
    pub assemblages: HashMap<String, Vec<Box<AssemblageComponent>>>,
    pub components: HashMap<ImString, Box<AssemblageComponent>>,
}
#[allow(dead_code)]
impl Assemblager {
    pub fn new() -> Assemblager {
        Self {
            assemblages: {
                let mut data = String::new();
                File::open(TYPES_PATH)
                    .unwrap()
                    .read_to_string(&mut data)
                    .unwrap();
                serde_json::from_str(&data).unwrap()
            },
            components: HashMap::new(),
        }
    }

    pub fn register_component<T: AssemblageComponent>(&mut self, component: T) {
        self.components
            .insert(ImString::new(component.name()), Box::new(component));
    }

    fn get_json(world: &specs::World, entity: Entity) -> String {
        use specs::Join;

        let mut output = String::new();
        output.push('[');

        let ps = world.read_resource::<PhysState>();
        for (phys, ent) in (&world.read_storage::<Phys>(), &world.entities()).join() {
            if ent == entity {
                let hitbox = ps.hitbox_from_phys(&phys);

                output.push_str("{\"Hitbox\":");
                output.push_str(&serde_json::to_string(&hitbox).unwrap());
                output.push_str("}");
            }
        }

        for storage in world.any_storages().iter(&world.res) {
            if let Some(Some(serialized_data)) = storage.serialize(entity) {
                output.push_str(",");
                output.push_str(&serialized_data);
            }
        }

        output.push(']');
        output
    }

    pub fn save_json(&self, lu: &LazyUpdate) {
        use specs::Join;

        let mut file = File::create(TYPES_PATH).unwrap();
        file.write_all(&serde_json::to_string(&self.assemblages).unwrap().as_bytes())
            .unwrap();

        lu.exec(move |world| {
            let mut serialized_entities = String::new();
            serialized_entities.push('[');
            let assemblaged = world.read_storage::<Assemblaged>();
            for (_, ent) in (&assemblaged, &world.entities()).join() {
                if serialized_entities.len() != 1 {
                    serialized_entities.push(',');
                }
                serialized_entities.push_str(&Self::get_json(&world, ent));
            }
            serialized_entities.push(']');
            let mut file = File::create(INSTANCES_PATH).unwrap();
            file.write_all(&serialized_entities.as_bytes()).unwrap();
        });
    }

    pub fn load_save(&self, world: &mut specs::World) {
        use specs::{Builder, Join};

        let mut file = File::open(INSTANCES_PATH).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        {
            let entity_data: Vec<Vec<Box<AssemblageComponent>>> =
                serde_json::from_str(&contents).unwrap();
            let lu = world.read_resource::<specs::world::LazyUpdate>();
            let ents = world.entities();

            for components in entity_data.iter() {
                let builder = lu.create_entity(&ents);

                for c in components {
                    c.add_to_lazy_builder(&builder);
                }

                builder.build();
            }
        }

        world.maintain();
        let mut appearance_builders = world.write_storage::<AppearanceBuilder>();
        for mut appear_builder in (&mut appearance_builders).join() {
            appear_builder.built = false;
        }
    }

    pub fn draft<'a, 'b, 'c>(
        &'a self,
        assemblage_key: &'c str,
        lu: &'b LazyUpdate,
        ents: &'a EntitiesRes,
    ) -> LazyBuilder<'b> {
        use specs::Builder;
        let components = &self.assemblages[assemblage_key];
        let mut builder = lu.create_entity(&ents);

        for c in components {
            c.add_to_lazy_builder(&builder);
        }

        builder = builder.with(Assemblaged {
            built_from: assemblage_key.to_string(),
        });

        builder
    }

    pub fn build<'a, 'b>(
        &self,
        assemblage_key: &str,
        lu: &'b LazyUpdate,
        ents: &'a EntitiesRes,
    ) -> Entity {
        use specs::Builder;
        self.draft(assemblage_key, lu, ents).build()
    }

    pub fn build_at<'a, 'b>(
        &self,
        assemblage_key: &str,
        lu: &'b LazyUpdate,
        ents: &'a EntitiesRes,
        pos: glm::TVec3<f32>,
    ) -> Entity {
        let e = self.build(assemblage_key, lu, ents);
        lu.exec(move |world| {
            let mut physes = world.write_storage::<Hitbox>();
            let mut phys = physes.get_mut(e).unwrap();
            phys.position = pos;
        });
        e
    }

    pub fn cache<'a>(
        &'a mut self,
        lazy_update: &'a LazyUpdate,
        entities_res: &'a EntitiesRes,
    ) -> Spawner<'a> {
        Spawner {
            assemblager: self,
            lazy_update,
            entities_res,
        }
    }
}

//ease of use thing so you don't have to pass LazyUpdate and EntitiesRes each and every time.
#[allow(dead_code)]
pub struct Spawner<'a> {
    assemblager: &'a mut Assemblager,
    lazy_update: &'a LazyUpdate,
    entities_res: &'a EntitiesRes,
}

#[allow(dead_code)]
impl<'a> Spawner<'a> {
    pub fn draft(&self, assemblage_key: &str) -> LazyBuilder {
        self.assemblager
            .draft(assemblage_key, self.lazy_update, self.entities_res)
    }

    pub fn build(&self, assemblage_key: &str) -> Entity {
        self.assemblager
            .build(assemblage_key, self.lazy_update, self.entities_res)
    }

    pub fn build_at(&self, assemblage_key: &str, pos: glm::TVec3<f32>) -> Entity {
        self.assemblager
            .build_at(assemblage_key, self.lazy_update, self.entities_res, pos)
    }
}
