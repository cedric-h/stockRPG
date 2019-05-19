use current::CurrentGuard;
use std::collections::HashMap;
use std::sync::Arc;

extern crate serde;

//SlitherData lets scripts allocate their own objects on the fly
//and these can even be saved for when the game is closed!
/*
#[derive(Default)]
struct SlitherData {
    cache_map: HashMap<specs::world::Index, slither::Object>,
    cache_refs: HashMap<specs::world::Index, usize>,
}*/

//this struct is exposed as a resource, it stores the data that's
//shoved into the slither console
#[derive(Default)]
pub struct SlitherConsole(pub String);

//this stores everything that's needed to run Slither code.
pub struct SlitherState {
    agent: slither::Agent,
    //module: std::sync::Arc<slither::Module>,
    //slither_data: SlitherData,
}
impl SlitherState {
    pub fn new() -> Self {
        use crate::prelude::*;
        use current::Current;
        use slither::Agent;
        use specs::{Join, LazyUpdate, World};

        let agent = Agent::new();

        //library functions

        //immediately move an entity somewhere
        fn teleport(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            //physics stuff
            let physes = world.read_storage::<Phys>();
            let mut ps = world.write_resource::<PhysState>();
            //entity stuff
            let ents = world.entities();
            let ent = ents.entity(rt.current_object::<u32>("entity")?);
            let coords = glm::make_vec3(&rt.pop_vec4::<[f32; 3]>()?);

            let phys = physes
                .get(ent)
                .ok_or("Teleport requested for non-physical entity.")?;

            ps.set_location(phys, &coords);

            Ok(())
        }
        module.add(
            Arc::new("teleport".into()),
            teleport,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::Vec4],
                ret: Type::Void,
            },
        );

        /*
        //get the cache of saveable slither data for a certain entity
        fn get_cache(rt: &mut Runtime) -> Result<(), String> {
            let mut slither_data = unsafe { Current::<SlitherData>::new() };
            let ent = rt.pop::<u32>()?;

            slither_data.cache_refs.insert(ent, rt.stack.len());
            let cache = slither_data
                .cache_map
                .entry(ent)
                .or_insert(slither::Object::default());
            rt.push(Variable::Object(Arc::clone(cache)));

            Ok(())
        }
        module.add(
            Arc::new("get_cache".into()),
            get_cache,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Object,
            },
        );

        //get the cache of saveable slither data for a certain entity
        fn save_caches(rt: &mut Runtime) -> Result<(), String> {
            let mut slither_data = unsafe { Current::<SlitherData>::new() };
            //consolidate the borrow, otherwise a new &mut ^ that will be made each time.
            let dd: &mut SlitherData = &mut *slither_data;

            //have to collect and iter over the drain because borrow checker! :D
            for (ent, index) in dd.cache_refs.drain() {
                match &rt.stack[index] {
                    Variable::Object(data) => {
                        dd.cache_map.insert(ent, Arc::clone(&data));
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

        //get an array of things with this scripting id
        slither_fn!{fn all_with_id(search_id: String) -> Vec<u32> {
            let world = unsafe { Current::<World>::new() };

            let ents = world.entities();
            let scripting_ids = world.read_storage::<ScriptingIds>();

            (&ents, &scripting_ids)
                .join()
                //find the entities whose list of scripting_ids contain search_id
                .filter(|(_, ScriptingIds { ids })| ids.contains(&search_id))
                //slither only deals with the id # of the entities, not the entity structs.
                .map(|(ent, _)| ent.id())
                //okay now vec that thing and ship it off
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

        //uses the assemblager to spawn an entity right next to another one.
        fn spawn_at_entity(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            //resources
            let ps = world.read_resource::<PhysState>();
            let assemblager = world.read_resource::<Assemblager>();
            let lu = world.read_resource::<LazyUpdate>();
            let ents = world.entities();

            //storages
            let physes = world.read_storage::<Phys>();

            //okay now get the entity, their position, and what to spawn
            let what_to_spawn = &rt.pop::<String>()?;
            let ent = ents.entity(rt.pop::<u32>()?);
            let pos = physes
                .get(ent)
                .and_then(|phys| ps.location(phys))
                .ok_or("can't spawn at an entity which has no position")?;

            assemblager.build_at(what_to_spawn, &lu, &ents, *pos);
            Ok(())
        }
        module.add(
            Arc::new("spawn_at_entity".into()),
            spawn_at_entity,
            Dfn {
                lts: vec![Lt::Default, Lt::Default],
                tys: vec![Type::F64, Type::Text],
                ret: Type::Void,
            },
        );

        //set health of an entity
        fn set_hp(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let ents = world.entities();
            let ent = ents.entity(rt.current_object::<u32>("entity")?);
            let mut health_storage = world.write_storage::<Health>();
            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            let health_value = rt.pop::<f32>()?;
            health.value = health.max.min(health_value);

            Ok(())
        }
        module.add(
            Arc::new("set_hp".into()),
            set_hp,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Void,
            },
        );

        //set health to a certain % of the current health
        fn set_hp_percent(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let ents = world.entities();
            let ent = ents.entity(rt.current_object::<u32>("entity")?);
            let mut health_storage = world.write_storage::<Health>();
            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            let percent = rt.pop::<f32>()?;
            health.value = health.max.min((percent / 100.0) * health.max);

            Ok(())
        }
        module.add(
            Arc::new("set_hp_percent".into()),
            set_hp_percent,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Void,
            },
        );

        //change health of an entity
        fn change_hp(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let ents = world.entities();
            let ent = ents.entity(rt.current_object::<u32>("entity")?);
            let mut health_storage = world.write_storage::<Health>();
            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            let health_value = rt.pop::<f32>()?;
            health.value += health.max.min(health.value + health_value);

            Ok(())
        }
        module.add(
            Arc::new("change_hp".into()),
            change_hp,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Void,
            },
        );

        //change health by a certain % of the current health
        fn change_hp_percent(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };

            let ents = world.entities();
            let ent = ents.entity(rt.current_object::<u32>("entity")?);
            let mut health_storage = world.write_storage::<Health>();
            let mut health = health_storage
                .get_mut(ent)
                .ok_or("Entity does not have health component")?;
            let percent = rt.pop::<f32>()?;
            health.value += health.max.min((percent / 100.0) * health.value);

            Ok(())
        }
        module.add(
            Arc::new("change_hp_percent".into()),
            change_hp_percent,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::F64],
                ret: Type::Void,
            },
        );

        //log a message into the Slither console in the DevUi
        slither_fn!{fn log(msg: String) {
            let world = unsafe { Current::<World>::new() };
            let mut slither_console = world.write_resource::<SlitherConsole>();

            //get the message they passed as an argument to the function
            slither_console.0.push_str(&format!("{}\n", msg));
        }}
        module.add(
            Arc::new("log".into()),
            log,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::Text],
                ret: Type::Void,
            },
        );*/

        //finally, return the instance with the filled module and runtime.
        Self {
            agent,
            //module: Arc::new(module),
            //slither_data: SlitherData::default(),
        }
    }

    pub fn run(&mut self) {
        use crate::prelude::*;
        use current::Current;
        use slither::Value;
        use specs::Join;

        //I'm fairly sure this world has to be dropped
        let script_events = {
            let world = unsafe { &*Current::<specs::World>::new() };
            let entities = world.entities();
            let mut event_storage = world.write_storage::<ScriptEvent>();
            (event_storage.drain(), &entities)
                .join()
                //we only want the ids for our purposes,
                .map(|(e, x)| (e, x.id()))
                .collect::<Vec<_>>()
                //copy so no references to the world remain!
                .clone()
        };
        let mut events_iter = script_events.iter().peekable();

        //if there's actually at least one event to bother cloning the module for...
        if events_iter.peek().is_some() {
            /*
            //manually split the borrow for the borrow checker
            let slither_data = &mut self.slither_data;
            let runtime = &mut self.runtime;

            //reload the code containing the function they want in case they changed it
            let mut module = (*self.module).clone();
            let module_load_error = load("src/sl/test.sl", &mut module)
                .err()
                //these are almost always syntax errors but I guess they could
                //be other things actually? whatever I'll just leave it this way.
                .map(|x| format!(" --- SYNTAX ERROR --- \n{}\n\n", x));
            let module = Arc::new(module);*/

            /*
            //open up the SlitherData for access by the scripts
            let slither_data_guard = CurrentGuard::new(slither_data);*/

            //a (potentially massive) string of the errors this thing could've outputted.
            let output = script_events
                .iter()
                //call each script_event's handler, and collect the errors
                //.map(|(_script_event, id)| {
                .map(|_| {
                    info!("here!");
                    let res = self.agent
                        .run("eval", &std::fs::read_to_string("src/sl/test.sl").unwrap())
                        .map_err(|x| Value::inspect(&self.agent, &x));
                    info!("holy shit it compiled into probably an error");
                    res
                })
                //now combine all of the errors into one,
                .fold(
                    //starting with the errors from loading the module,
                    //or an empty string if there were none,
                    String::new(),
                    //then for each script event that was run,
                    //add its error too if it emitted one.
                    |mut acc, res| {
                        match res {
                            //we only care about the errors
                            Err(err) => {
                                info!("finna bouta push the error to the output string");
                                acc.push_str(&format!(" --- ERROR --- \n{}\n\n", err));
                                info!("if you don't get this message Display prolly got recurisve");
                            }
                            _ => {}
                        };
                        acc
                    },
                );

            /*
            //now that all of the modules are done accessing it,
            drop(slither_data_guard);*/

            //quickly add the errors that could've been outputted to the console
            let world = unsafe { &*Current::<specs::World>::new() };
            let mut slither_console = world.write_resource::<SlitherConsole>();
            slither_console.0.push_str(&output);
        }
    }
}

//this is the fancy little struct that stores all of the data
//for the scripts. It's formatted this way so that the data can
//also be saved, when that needs to occur.

/*
#[derive(Default)]
pub struct SlitherCache {
    pub data: slither::Object,
}

use serde::ser::{Error, Serialize, SerializeMap, Serializer};
impl Serialize for SlitherCache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use slither::Variable::*;

        let mut map = serializer.serialize_map(Some(self.data.len()))?;

        for (k, v) in self.data.iter() {
            map.serialize_entry(
                &**k,
                &(match v {
                    Text(slither_string) => Ok(slither_string.to_string()),
                    F64(slither_num, _) => Ok(slither_num.to_string()),
                    Bool(slither_bool, _) => Ok(slither_bool.to_string()),
                    _ => Err(S::Error::custom(
                        "The game can't save a variable of that type!",
                    )),
                }?),
            )?;
        }

        map.end()
    }
}*/
