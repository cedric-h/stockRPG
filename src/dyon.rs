use current::CurrentGuard;
use dyon::Variable;
use std::collections::HashMap;
use std::sync::Arc;

extern crate serde;

// DyonData lets scripts allocate their own objects on the fly
// and these can even be saved for when the game is closed!
#[derive(Default)]
struct DyonData {
    cache_map: HashMap<specs::world::Index, dyon::Object>,
    cache_refs: HashMap<specs::world::Index, usize>,
}

// this struct is exposed as a resource, it stores the data that's
// shoved into the dyon console
#[derive(Default)]
pub struct DyonConsole(pub String);

// this stores everything that's needed to run Dyon code.
pub struct DyonState {
    runtime: dyon::Runtime,
    module: std::sync::Arc<dyon::Module>,
    dyon_data: DyonData,
}
impl DyonState {
    pub fn new() -> Self {
        use crate::prelude::*;
        use current::Current;
        use dyon::{dyon_fn, dyon_fn_pop, dyon_macro_items, Dfn, Lt, Module, Runtime, Type};
        use specs::{Join, LazyUpdate, World};

        let runtime = Runtime::new();

        let mut module = Module::new();

        // library functions

        // immediately move an entity somewhere
        fn teleport_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let coord_arr: [f32; 3] = rt.pop_vec4()?;
            let ent_id: u32 = rt.pop()?;

            // physics stuff
            let physes = world.read_storage::<Phys>();
            let mut ps = world.write_resource::<PhysState>();
            // entity stuff
            let ents = world.entities();
            let coords = glm::make_vec3(&coord_arr);
            let ent = ents.entity(ent_id);

            let phys = physes
                .get(ent)
                .ok_or("Teleport requested for non-physical entity.")?;

            ps.set_location(phys, &coords);

            Ok(())
        }
        module.add(
            Arc::new("teleport_entity".into()),
            teleport_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::Vec4],
                ret: Type::Void,
            },
        );

        // get the cache of saveable dyon data for a certain entity
        fn get_cache_entity(rt: &mut Runtime) -> Result<(), String> {
            let mut dyon_data = unsafe { Current::<DyonData>::new() };
            let ent = rt.pop::<u32>()?;

            dyon_data.cache_refs.insert(ent, rt.stack.len());
            let cache = dyon_data
                .cache_map
                .entry(ent)
                .or_insert(dyon::Object::default());
            rt.push(Variable::Object(Arc::clone(cache)));

            Ok(())
        }
        module.add(
            Arc::new("get_cache_entity".into()),
            get_cache_entity,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Object,
            },
        );

        // get the cache of saveable dyon data for a certain entity
        fn save_caches(rt: &mut Runtime) -> Result<(), String> {
            let mut dyon_data = unsafe { Current::<DyonData>::new() };

            // have to collect and iter over the drain because borrow checker! :D
            for (ent, index) in dyon_data.cache_refs.drain().collect::<Vec<_>>().iter() {
                match &rt.stack[*index] {
                    Variable::Object(data) => {
                        dyon_data.cache_map.insert(*ent, Arc::clone(data));
                        Ok(())
                    }
                    _ => Err(
                        "Found the wrong type of variable where the cache should be!".to_owned(),
                    ),
                }?
            }

            Ok(())
        }
        module.add(
            Arc::new("save_caches".into()),
            save_caches,
            Dfn {
                lts: vec![],
                tys: vec![],
                ret: Type::Void,
            },
        );

        // get an array of things with this scripting id
        dyon_fn! {fn all_with_id(search_id: String) -> Vec<u32> {
            let world = unsafe { Current::<World>::new() };

            let ents = world.entities();
            let scripting_ids = world.read_storage::<ScriptingIds>();

            (&ents, &scripting_ids)
                .join()
                // find the entities whose list of scripting_ids contain search_id
                .filter(|(_, ScriptingIds { ids })| ids.contains(&search_id))
                // dyon only deals with the id # of the entities, not the entity structs.
                .map(|(ent, _)| ent.id())
                // okay now vec that thing and ship it off
                .collect::<Vec<_>>()
        }}
        module.add(
            Arc::new("all_with_id".into()),
            all_with_id,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::Text],
                ret: Type::Array(Box::new(Type::F64)),
            },
        );

        fn add_id_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };
            let ents = world.entities();
            let mut scripting_ids = world.write_storage::<ScriptingIds>();

            let scripting_id: String = rt.pop()?;
            let ent_id: u32 = rt.pop()?;

            let ent = ents.entity(ent_id);
            let ent_ids = scripting_ids
                .get_mut(ent)
                .ok_or("That entity doesn't exist, or doesn't/can't have scripting ids.")?;
            ent_ids.ids.push(scripting_id);
            Ok(())
        }
        module.add(
            Arc::new("add_id_entity".into()),
            add_id_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::Text],
                ret: Type::Void,
            },
        );

        fn has_id_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };
            let ents = world.entities();
            let scripting_ids = world.read_storage::<ScriptingIds>();

            let scripting_id: String = rt.pop()?;
            let ent_id: u32 = rt.pop()?;

            let ent = ents.entity(ent_id);
            match scripting_ids.get(ent) {
                Some(ent_ids) => rt.push(ent_ids.ids.contains(&scripting_id)),
                None => rt.push(false),
            }
            Ok(())
        }
        module.add(
            Arc::new("has_id_entity".into()),
            has_id_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::Text],
                ret: Type::Bool,
            },
        );

        fn delete_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { &mut *Current::<World>::new() };

            // I want all of these to be dropped before I do world.maintain()
            {
                let ent_id: u32 = rt.pop()?;

                let entities = world.entities();

                entities.delete(entities.entity(ent_id)).unwrap();
            }

            world.maintain();

            Ok(())
        }
        module.add(
            Arc::new("delete_entity".into()),
            delete_entity,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Void,
            },
        );

        // uses the assemblager to spawn an entity right next to another one.
        fn spawn_at_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { &mut *Current::<World>::new() };

            {
                // okay now get the entity, their position, and what to spawn
                let what_to_spawn: String = rt.pop()?;
                let ent_id: u32 = rt.pop()?;

                // resources
                let ps = world.read_resource::<PhysState>();
                let assemblager = world.read_resource::<Assemblager>();
                let lu = world.read_resource::<LazyUpdate>();
                let ents = world.entities();
                let physes = world.read_storage::<Phys>();

                // action
                let ent = ents.entity(ent_id);
                let pos = physes
                    .get(ent)
                    .and_then(|phys| ps.location(phys))
                    .ok_or("can't spawn at an entity which has no position")?;

                rt.push(assemblager.build_at(&what_to_spawn, &lu, &ents, *pos).id());
            }

            world.maintain();
            Ok(())
        }
        module.add(
            Arc::new("spawn_at_entity".into()),
            spawn_at_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::Text],
                ret: Type::F64,
            },
        );

        // set health of an entity
        fn set_hp_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let health_value: f32 = rt.pop()?;
            let ent_id: u32 = rt.pop()?;

            let ents = world.entities();
            let ent = ents.entity(ent_id);
            let mut health_storage = world.write_storage::<Health>();

            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            health.value = health.max.min(health_value);

            Ok(())
        }
        module.add(
            Arc::new("set_hp_entity".into()),
            set_hp_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::F64],
                ret: Type::Void,
            },
        );

        // set health to a certain % of the current health
        fn set_hp_percent_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let percent: f32 = rt.pop()?;
            let ent_id: u32 = rt.pop()?;

            let ents = world.entities();
            let ent = ents.entity(ent_id);
            let mut health_storage = world.write_storage::<Health>();

            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;

            health.value = health.max.min((percent / 100.0) * health.max);

            Ok(())
        }
        module.add(
            Arc::new("set_hp_percent_entity".into()),
            set_hp_percent_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::F64],
                ret: Type::Void,
            },
        );

        // change health of an entity
        fn change_hp_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let health_value = rt.pop::<f32>()?;
            let ent_id: u32 = rt.pop()?;

            let ents = world.entities();
            let mut health_storage = world.write_storage::<Health>();
            let ent = ents.entity(ent_id);
            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            health.value += health.max.min(health.value + health_value);

            Ok(())
        }
        module.add(
            Arc::new("change_hp_entity".into()),
            change_hp_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::F64],
                ret: Type::Void,
            },
        );

        // change health by a certain % of the current health
        fn change_hp_percent_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let percent: f32 = rt.pop()?;
            let ent_id: u32 = rt.pop()?;

            let ents = world.entities();
            let ent = ents.entity(ent_id);
            let mut health_storage = world.write_storage::<Health>();
            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            health.value += health.max.min((percent / 100.0) * health.value);

            Ok(())
        }
        module.add(
            Arc::new("change_hp_percent_entity".into()),
            change_hp_percent_entity,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Void,
            },
        );

        // log a message into the Dyon console in the DevUi
        dyon_fn! {fn log(msg: String) {
            let world = unsafe { Current::<World>::new() };
            let mut dyon_console = world.write_resource::<DyonConsole>();

            // get the message they passed as an argument to the function
            dyon_console.0.push_str(&format!("{}\n", msg));
        }}
        module.add(
            Arc::new("log".into()),
            log,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::Text],
                ret: Type::Void,
            },
        );

        // finally, return the instance with the filled module and runtime.
        Self {
            runtime,
            module: Arc::new(module),
            dyon_data: DyonData::default(),
        }
    }

    pub fn run(&mut self) {
        use crate::prelude::*;
        use current::Current;
        use dyon::{load, Call};
        use specs::Join;

        // I'm fairly sure this world has to be dropped
        let script_events = {
            let world = unsafe { &*Current::<specs::World>::new() };
            let entities = world.entities();
            let mut event_storage = world.write_storage::<ScriptEvent>();
            (event_storage.drain(), &entities)
                .join()
                // we only want the ids for our purposes,
                .map(|(e, x)| (e, x.id()))
                .collect::<Vec<_>>()
                // copy so no references to the world remain!
                .clone()
        };
        let mut events_iter = script_events.iter().peekable();

        // if there's actually at least one event to bother cloning the module for...
        if events_iter.peek().is_some() {
            // manually split the borrow for the borrow checker
            let dyon_data = &mut self.dyon_data;
            let runtime = &mut self.runtime;

            // reload the code containing the function they want in case they changed it
            let mut module = (*self.module).clone();
            let module_load_error = load("src/dyon/test.dyon", &mut module)
                .err()
                // these are almost always syntax errors but I guess they could
                // be other things actually? whatever I'll just leave it this way.
                .map(|x| format!(" --- SYNTAX ERROR --- \n{}\n\n", x));
            let module = Arc::new(module);

            // open up the DyonData for access by the scripts
            let dyon_data_guard = CurrentGuard::new(dyon_data);

            // output
            let output = script_events
                .iter()
                // call each script_event's handler, and collect the errors
                .map(|(script_event, id)| {
                    let event_handler = Call::new(&script_event.function)
                        .arg(*id)
                        .arg(script_event.payload);
                    event_handler
                        .run(runtime, &Arc::clone(&module))
                        .map_err(|err| format!("fn {}: \n {}", script_event.function, err))
                })
                // now combine all of the errors into one,
                .fold(
                    // starting with the errors from loading the module,
                    // or an empty string if there were none,
                    module_load_error.unwrap_or(String::new()),
                    // then for each script event that was run,
                    // add its error too if it emitted one.
                    |mut acc, res| {
                        match res {
                            // we only care about the errors
                            Err(err) => {
                                acc.push_str(&format!(" --- ERROR --- \n{}\n\n", err));
                            }
                            _ => {}
                        };
                        acc
                    },
                );

            // now that all of the modules are done accessing it,
            drop(dyon_data_guard);

            // quickly add the errors that could've been outputted to the console
            let world = unsafe { &*Current::<specs::World>::new() };
            let mut dyon_console = world.write_resource::<DyonConsole>();
            dyon_console.0.push_str(&output);
        }
    }
}

// this is the fancy little struct that stores all of the data
// for the scripts. It's formatted this way so that the data can
// also be saved, when that needs to occur.

/*
#[derive(Default)]
pub struct DyonCache {
    pub data: dyon::Object,
}

use serde::ser::{Error, Serialize, SerializeMap, Serializer};
impl Serialize for DyonCache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use dyon::Variable::*;

        let mut map = serializer.serialize_map(Some(self.data.len()))?;

        for (k, v) in self.data.iter() {
            map.serialize_entry(
                &**k,
                &(match v {
                    Text(dyon_string) => Ok(dyon_string.to_string()),
                    F64(dyon_num, _) => Ok(dyon_num.to_string()),
                    Bool(dyon_bool, _) => Ok(dyon_bool.to_string()),
                    _ => Err(S::Error::custom(
                        "The game can't save a variable of that type!",
                    )),
                }?),
            )?;
        }

        map.end()
    }
}*/
