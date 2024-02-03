use std::process::Command;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    gtk::{
        self,
        prelude::{ApplicationExt, EditableExt, EntryBufferExtManual, EntryExt, WidgetExt},
    },
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt,
};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};

struct AppModel {
    entry_id: gtk::EntryBuffer,
    entry_zip: gtk::EntryBuffer,
    entry_name: gtk::EntryBuffer,
    show_button: bool,
    show_indicator: bool,
    dialog_success: Controller<Alert>,
    dialog_failure: Controller<Alert>,
}

#[derive(Debug)]
enum AppInMsg {
    Start,
    Recheck,
    Confirm,
    Cancel,
    Option,
}

#[derive(Debug)]
enum CommandMsg {
    Data(bool),
}

#[relm4::component]
impl Component for AppModel {
    type Input = AppInMsg;
    type CommandOutput = CommandMsg;

    type Output = ();
    type Init = u8;

    // Initialize the UI.
    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            entry_id: gtk::EntryBuffer::new(Some("v123")),
            entry_zip: gtk::EntryBuffer::new(Some("dict.zip")),
            entry_name: gtk::EntryBuffer::new(Some("VNDB - Names")),
            show_button: true,
            show_indicator: false,
            // status_code: None,
            dialog_success: Alert::builder()
                .transient_for(root)
                .launch(AlertSettings {
                    text: String::from("Deck Created Successfully"),
                    secondary_text: None,
                    confirm_label: String::from("Quit"),
                    cancel_label: String::from("Dismiss"),
                    option_label: None,
                    is_modal: true,
                    destructive_accept: false,
                })
                .forward(sender.input_sender(), convert_alert_response),
            dialog_failure: Alert::builder()
                .transient_for(root)
                .launch(AlertSettings {
                    text: String::from("Deck Creation Failed"),
                    secondary_text: Some(String::from(
                        "Please raise an issue if this happens consistently",
                    )),
                    confirm_label: String::from("Quit"),
                    cancel_label: String::from("Dismiss"),
                    option_label: None,
                    is_modal: true,
                    destructive_accept: false,
                })
                .forward(sender.input_sender(), convert_alert_response),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        match msg {
            AppInMsg::Start => {
                self.show_button = false;
                self.show_indicator = true;
                let a = self.entry_id.text().to_string().clone();
                let b = self.entry_zip.text().to_string().clone();
                let c = self.entry_name.text().to_string().clone();
                sender
                    .oneshot_command(async move { CommandMsg::Data(execute_js(&a, &b, &c).await) });
            }
            AppInMsg::Recheck => {
                self.show_button = self.entry_id.length() != 0
                    && self.entry_zip.length() != 0
                    && self.entry_name.length() != 0;
            }
            AppInMsg::Option => {}
            AppInMsg::Confirm => {
                //TODO quit the app
                relm4::main_application().quit();
            }
            AppInMsg::Cancel => {
                //TODO dismiss
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            CommandMsg::Data(success) => {
                self.show_button = true;
                self.show_indicator = false;
                if success {
                    self.dialog_success.emit(AlertMsg::Show);
                } else {
                    self.dialog_failure.emit(AlertMsg::Show);
                }
            }
        }
    }

    view! {
        gtk::Window {
            set_title: Some("VNDB Characters to Yomitan"),
            set_default_width: 600,
            set_default_height: 200,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Box {
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Label {
                        set_label: "VNDB ID"

                    },
                    gtk::Entry {
                        set_buffer: &model.entry_id,
                        set_tooltip_text: Some("ID of the VN on VNDB"),
                        connect_changed => AppInMsg::Recheck,
                    },
                },

                gtk::Box {
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Label {
                        set_label: "Output zip file"

                    },
                    gtk::Entry {
                        set_buffer: &model.entry_zip,
                        set_tooltip_text: Some("zip file the dictionary will be saved to"),
                        connect_changed => AppInMsg::Recheck,

                    },
                },

                gtk::Box {
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Label {
                        set_label: "Name of the dictionary as desplayed in Yomitan"

                    },
                    gtk::Entry {
                        set_buffer: &model.entry_name,
                        set_tooltip_text: Some("Name of the dictionary in Yomitan"),
                        connect_changed => AppInMsg::Recheck,

                    },
                },

                append = if model.show_button {
                gtk::Button::with_label("Generate Deck !") {
                    connect_clicked[sender] => move |_| {
                        sender.input(AppInMsg::Start);
                    }
                }}
                else if model.show_indicator {
                    gtk::Spinner {
                        set_spinning: true,
                    }
                }
                else {
                    gtk::Label{
                        set_label: "Please fill all three fields"
                    }
                },

            }
        }
    }
}

pub async fn execute_js(entry_id: &str, entry_zip: &str, entry_name: &str) -> bool {
    // if cfg!(target_os = "windows") {
    //     Command::new("cmd")
    //         .args(["/C", "echo hello"])
    //         .output()
    //         .expect("failed to execute process")
    // } else {
    //
    let a = Command::new("./yomitan")
        .arg(entry_id)
        .arg(entry_zip)
        .arg(entry_name)
        .status()
        .expect("failed to execute process");
    // };
    a.success()
}

fn convert_alert_response(response: AlertResponse) -> AppInMsg {
    match response {
        AlertResponse::Confirm => AppInMsg::Confirm,
        AlertResponse::Cancel => AppInMsg::Cancel,
        AlertResponse::Option => AppInMsg::Option,
    }
}

fn main() {
    let app = RelmApp::new("relm4.test.simple");
    app.run::<AppModel>(0);
}
