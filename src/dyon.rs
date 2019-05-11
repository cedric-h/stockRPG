use crate::prelude::*;

#[derive(Default)]
pub struct DyonConsole(pub String);

pub struct DyonState {
    runtime: dyon::Runtime,
    module: std::sync::Arc<dyon::Module>,
}
impl DyonState {
    pub fn new() -> Self {
        use current::Current;
        use dyon::{Dfn, Lt, Module, Runtime, Type};
        use specs::World;
        use std::sync::Arc;

        let mut module = Module::new();

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

        //log a message into the Dyon console in the DevUi
        fn log(rt: &mut Runtime) -> Result<(), String> {
            let world = unsafe { Current::<World>::new() };
            let mut dyon_console = world.write_resource::<DyonConsole>();

            //get the message they passed as an argument to the function
            let message: Arc<String> = rt.pop()?;
            dyon_console.0.push_str(&format!("{}\n", &**message));
            Ok(())
        }
        module.add(
            Arc::new("log".into()),
            log,
            Dfn {
                lts: vec![Lt::Default],
                tys: vec![Type::Text],
                ret: Type::Void,
            },
        );

        //finally, return the instance with the filled module and runtime.
        Self {
            runtime: Runtime::new(),
            module: Arc::new(module),
        }
    }

    pub fn run(&mut self) {
        use current::Current;
        use dyon::{load, Call};
        use specs::Join;
        use std::sync::Arc;

        //I'm fairly sure this world has to be dropped
        let script_events = {
            let world = unsafe { &*Current::<specs::World>::new() };
            let entities = world.entities();
            let mut event_storage = world.write_storage::<ScriptEvent>();
            (event_storage.drain(), &entities)
                .join()
                .map(|(e, x)| (e, x.id()))
                .collect::<Vec<_>>()
                .clone()
        };
        let mut events_iter = script_events.iter().peekable();

        //if there's actually at least one event to bother cloning the module for...
        if events_iter.peek().is_some() {
            //reload the code containing the function they want in case they changed it
            let mut module = (*self.module).clone();
            let module_load_error = load("src/dyon/test.dyon", &mut module)
                .err()
                .map(|x| format!(" --- SYNTAX ERROR --- \n{}\n\n", x));
            let module = Arc::new(module);

            //output
            let output = script_events
                .iter()
                .map(|(script_event, id)| {
                    let event_handler = Call::new(&script_event.function).arg(*id);
                    event_handler.run(&mut self.runtime, &Arc::clone(&module))
                })
                .fold(
                    module_load_error.unwrap_or(String::new()),
                    |mut acc, res| {
                        match res {
                            Err(err) => {
                                acc.push_str(&format!(" --- ERROR --- \n{}\n\n", err));
                            }
                            _ => {}
                        };
                        acc
                    },
                );

            let world = unsafe { &*Current::<specs::World>::new() };
            let mut dyon_console = world.write_resource::<DyonConsole>();
            dyon_console.0.push_str(&output);
        }
    }
}
