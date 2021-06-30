use druid::piet::{Text, TextAlignment, TextLayoutBuilder, TextAttribute};
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Flex};
use druid::{
    commands, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, FileDialogOptions,
    FileSpec, Handled, Lens, LocalizedString, Selector, Target, Widget,
    WindowDesc, FontWeight
};
use std::{
    thread,
    time::{Duration, Instant},
};

struct Delegate;

#[derive(Clone, Data, Lens)]
struct WindowState {
    text: String,
    color: Color,
    file_path: Option<String>,
}

struct RFIDChecker {
    txt_contents: String,
    is_unique: bool,
    len: usize
}

const UPDATED_FILE: Selector<Color> = Selector::new("rfid-checker.saved-file");
static mut FILE_PATH: Option<String> = None;

impl std::fmt::Display for WindowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl RFIDChecker {
    fn new() -> Self {
        Self {
            txt_contents: "".to_string(),
            is_unique: true,
            len: 0
        }
    }

    fn set_content(&mut self, txt: String) {
        self.txt_contents = txt
    }
    fn check_unique(&mut self) -> bool {
        let text_iter: Vec<&str> = self.txt_contents.split_whitespace().collect();
        let rfids: Vec<&str> = text_iter
            .iter()
            .filter(|string| string.contains("DC") && string.len() == 10)
            .cloned()
            .collect();

        self.len = rfids.len();
        self.is_unique = !(1..rfids.len()).any(|i| rfids[i..].contains(&rfids[i - 1]));
        self.is_unique
    }

    unsafe fn run_every_sec(&mut self, event_sink: druid::ExtEventSink) {
        let wait_time = Duration::from_millis(1000);
        loop {
            let start = Instant::now();
            if let Some(path) = &FILE_PATH {
                if path.len() > 0 {
                    match std::fs::read_to_string(path) {
                        Ok(s) => {
                            let text = s.to_string();
                            self.set_content(text);

                            let mut color = match self.check_unique() {
                                true => Color::GREEN,
                                false => Color::RED,
                            };

                            if self.len == 0 { 
                                color = Color::BLUE;
                            }

                            if event_sink
                                .submit_command(UPDATED_FILE, color.clone(), Target::Auto)
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("Error opening file: {}", e);
                        }
                    }
                }
            }

            let runtime = start.elapsed();
            if let Some(remaining) = wait_time.checked_sub(runtime) {
                thread::sleep(remaining);
            }
        }
    }
}

pub fn main() {
    let mut app = RFIDChecker::new();

    let main_window = WindowDesc::new(ui_builder())
        .title(
            LocalizedString::new("rfid-uniq-check")
                .with_placeholder("RFID Duplicates Checker"),
        )
        .window_size((700.0, 500.0));

    let initial_state = WindowState {
        text: "Otvor TXT súbor, do ktorého\nsa zapisujú RFID dáta.".to_string(),
        color: Color::BLACK,
        file_path: None,
    };

    let launcher = AppLauncher::with_window(main_window);

    let event_sink = launcher.get_external_handle();

    thread::spawn(move || unsafe { app.run_every_sec(event_sink) });

    launcher
        .delegate(Delegate)
        .launch(initial_state)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<WindowState> {
    let txt = FileSpec::new("Text file", &["txt"]);
    let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![txt])
        .default_type(txt)
        .clone()
        .name_label("TXT subor")
        .title("Vyber TXT subor s RFID datami")
        .button_text("Spusti program");

    // let input = TextBox::new();
    let open = Button::new("Otvor TXT súbor...").on_click(move |ctx, _, _| {
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options.clone()))
    });

    let mut col = Flex::column();
    col.add_child(StatusPanel);
    col.add_spacer(8.0);
    col.add_child(open);
    Align::centered(col)
}

/// A widget that displays a color.
struct StatusPanel;

impl Widget<WindowState> for StatusPanel {
    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, data: &mut WindowState, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.is(UPDATED_FILE) => {
                // update causes repaint
                data.color = cmd.get_unchecked(UPDATED_FILE).clone();
                
                if data.color == Color::GREEN {
                    data.text = "Všetky RFIDs sú unikátne.\nvýborná práca!".to_string();
                } else if data.color == Color::RED {
                    data.text = "NAŠLI SA\nDUPLIKÁTY!!!".to_string();
                } else {
                    data.text = "Textový súbor zatiaľ\nneobsahuje RFID dáta.".to_string();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &WindowState,
        _: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &WindowState, data: &WindowState, _: &Env) {
        if old_data.color != data.color {
            ctx.request_paint()
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &WindowState, _: &Env) -> Size {
        if bc.is_width_bounded() | bc.is_height_bounded() {
            let size = Size::new(600.0, 300.0);
            bc.constrain(size)
        } else {
            bc.max()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &WindowState, _env: &Env) {
        let rect = ctx.size().to_rounded_rect(5.0);
        ctx.fill(rect, &data.color);

        let text = ctx.text();
        let helvetica = text.font_family("Arial").unwrap();
        let layout = text
            .new_text_layout(data.text.clone())
            .default_attribute(TextAttribute::FontFamily(helvetica))
            .default_attribute(TextAttribute::FontSize(30.0))
            .default_attribute(TextAttribute::Weight(FontWeight::BOLD))
            .text_color(Color::WHITE)
            .alignment(TextAlignment::default())
            .build()
            .unwrap();
        ctx.draw_text(&layout, (100.0, 120.0));
    }
}

impl AppDelegate<WindowState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut WindowState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            if let Some(path) = file_info.path().to_str() {
                unsafe {
                    FILE_PATH = Some(String::from(path));
                }
                data.file_path = Some(String::from(path));
            }
            return Handled::Yes;
        }
        Handled::No
    }
}
