use super::prelude::*;

//scripting components

#[derive(
    Default,
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Clone,
    Debug,
)]
#[storage(VecStorage)]
pub struct Interactable {
    pub message: String,
    pub script: ScriptEvent,
}
impl DevUiRender for Interactable {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Interactable"));

        let mut im_message = ImString::with_capacity(100); //self.message.len() + 1);
        im_message.push_str(&self.message);
        if ui
            .input_text_multiline(im_str!("< Message"), &mut im_message, [300.0, 45.0])
            .build()
        {
            self.message = im_message.to_str().to_owned();
        }

        self.script.dev_ui_render(ui, world);
        //dbg!(self.message.as_bytes());
    }
}

#[derive(
    Default,
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Debug,
    Clone,
)]
#[storage(HashMapStorage)]
pub struct ScriptEvent {
    pub function: String,
}
impl DevUiRender for ScriptEvent {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("ScriptEvent"));

        ui.text(im_str!("function name: "));

        let mut im_function = ImString::with_capacity(100); //self.message.len() + 1);
        im_function.push_str(&self.function);
        if ui
            .input_text(im_str!("function name input"), &mut im_function)
            .build()
        {
            self.function = im_function.to_str().to_owned();
        }
    }
}

#[derive(
    Default,
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Clone,
    Debug,
)]
#[storage(VecStorage)]
pub struct ScriptingIds {
    pub ids: Vec<String>,
}
impl DevUiRender for ScriptingIds {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Scripting Identifiers"));

        let mut active_id_im_string = ImString::with_capacity(25);
        if let Some(id) = self.ids.last() {
            info!("pushing {}", id);
            active_id_im_string.push_str(id);
        }

        //this can add an id, or change an existing one.
        if ui
            .input_text(im_str!("< Id to change"), &mut active_id_im_string)
            .build()
        {
            let edited_id: String = active_id_im_string.to_str().into();

            //if it has been erased, no point in doing anything
            if edited_id.len() > 0 {

                //if they're changing the last one, remove the last one
                if self.ids.last().is_some() {
                    self.ids.pop();
                }

                self.ids.push(edited_id);
            }

            //fix this bug where we keep getting an event if the input is empty
            //by making sure things can only get deleted if the event is actually real
            else if let Some(id) = self.ids.last() {
                if id.len() == 1 && edited_id.len() == 0 {
                    self.ids.pop();
                }
            }
        }

        //This adds a new id
        if ui.button(im_str!("New"), [85.0, 20.0]) {
            self.ids.push("new id".into());
        }

        //these things can only be done if there's at least one thing
        if let Some(id) = self.ids.last() {
            ui.same_line(85.0 + 15.0);

            //remove the last one if they press remove or the last one's length is 0
            dbg!(id.len());
            if ui.button(im_str!("Remove"), [85.0, 20.0]) {
                info!("popped off {:?}", self.ids.pop());
            }

            //make names for them to click and move the name to the end if they do
            if let Some((index, _)) = self.ids.iter().enumerate().find(|(index, id)| {
                ui.selectable(
                    im_str!("{}", id),
                    *index == self.ids.len(),
                    ImGuiSelectableFlags::empty(),
                    ImVec2::new(0.0, 0.0),
                )
            }) {
                let clicked = self.ids.remove(index);
                self.ids.push(clicked);
            }
        }
    }
}
