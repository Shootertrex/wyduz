#![windows_subsystem = "windows"]
mod controller;

use controller::{make_request, HttpMethod, Request};
use fltk::{
    button::Button,
    enums::*,
    frame::Frame,
    group::{Flex, Tabs, Tile},
    prelude::*,
    utils::oncelock::Lazy,
    window::Window,
    *,
};
use fltk_theme::{ThemeType, WidgetTheme};
use std::str::FromStr;
use std::time::Instant;

static STATE: Lazy<app::GlobalState<State>> = Lazy::new(app::GlobalState::<State>::get);

pub struct State {
    pub url: String,
    pub method: String,
    pub request_buffer: text::TextBuffer,
    pub response_buffer: text::TextBuffer,
    pub header_tab: Flex,
    pub request_headers: Vec<(input::Input, input::Input)>,
    pub response_headers: Flex,
    pub status_code: frame::Frame,
    // pub saved: bool,
    // pub current_file: PathBuf,
}

impl State {
    fn new(
        request_buffer: text::TextBuffer,
        response_buffer: text::TextBuffer,
        header_tab: Flex,
        response_headers: Flex,
        status_code: frame::Frame,
    ) -> Self {
        State {
            url: String::new(),
            method: String::from("GET"),
            request_buffer,
            response_buffer,
            header_tab,
            request_headers: Vec::new(),
            response_headers,
            status_code,
            // saved: true,
            // current_file: PathBuf::new(),
        }
    }
}

fn send_cb(_e: &mut Button) {
    STATE.with(|s| {
        let mut headers = Vec::new();
        for header in s.request_headers.iter() {
            headers.push((header.0.value(), header.1.value()));
        }
        let request = Request {
            method: HttpMethod::from_str(&s.method).expect("invalid method"),
            url: s.url.clone(),
            headers,
            body: Some(s.request_buffer.text()),
        };

        let start = Instant::now();
        match make_request(&request) {
            Ok(response) => {
                let duration = start.elapsed();
                println!("response: {:?}\n{:?}", response, duration);
                s.response_buffer.set_text(&response.body);
                s.response_headers.clear();
                build_response_header_ui(&mut s.response_headers, &response.headers);
                s.status_code.set_label(&response.status_code.to_string());
            }
            Err(e) => {
                let duration = start.elapsed();
                // TODO: check if there was a response to display. 400s and 500s come back as
                // errors but are still valid responses.
                println!("Error making request: {}\n{:?}", e, duration);
                s.response_headers.clear();
                s.response_buffer.set_text("");
            }
        }
    });
}

fn url_input_cb(e: &mut input::Input) {
    let url = e.value();
    STATE.with(move |s| {
        s.url = url.clone();
    });
}

fn method_picker_cb(e: &mut menu::Choice) {
    let method = e.choice().expect("why didn't you choose something?");
    STATE.with(move |s| {
        s.method = method.clone();
    });
}

// TODO: make require the parent component as param. Then try to see if nested tiles can be made
// without having parent tiles 'erase' children ones. That might make adding url bar that stretches
// across the whole app easier.
fn build_tile() -> Tile {
    let dx = 20;
    let dy = dx;
    let tile = Tile::default_fill();
    let r = Frame::new(
        tile.x() + dx,
        tile.y() + dy,
        tile.w() - 2 * dx,
        tile.h() - 2 * dy,
        None,
    );
    tile.resizable(&r);

    tile
}

fn build_response_header_ui(response_header_flex: &mut Flex, headers: &[(String, String)]) {
    for header in headers.iter() {
        let header_flex = Flex::default_fill().row();
        let mut key = input::Input::default();
        key.set_value(&header.0);
        key.set_readonly(true);
        key.set_position(0)
            .expect("failed to move response header input to index 0");
        let mut value = input::Input::default();
        value.set_value(&header.1);
        value.set_readonly(true);
        value
            .set_position(0)
            .expect("failed to move response header input to index 0");
        header_flex.end();

        response_header_flex.add(&header_flex);
        response_header_flex.fixed(&header_flex, 30);
    }
}

fn build_request_headers_tab() -> Flex {
    let mut request_headers_tab = Flex::default_fill().column().with_label("Headers\t");
    let mut add_button = Button::default().with_label("Add");
    add_button.set_callback(|_c| {
        STATE.with(move |s| {
            let header_row_flex = Flex::default_fill().row();
            let key = input::Input::default();
            let value = input::Input::default();
            header_row_flex.end();
            // Need to add the component to the Flex first and only then can its size be set.
            s.header_tab.add(&header_row_flex);
            s.header_tab.fixed(&header_row_flex, 30);
            s.request_headers.push((key, value));
        });
    });
    request_headers_tab.fixed(&add_button, 30);
    request_headers_tab.add(&add_button);
    request_headers_tab.end();

    request_headers_tab
}

fn build_request_section(
    x: i32,
    parent_w: i32,
    parent_h: i32,
    request_buffer: text::TextBuffer,
    request_headers_tab: Flex,
) -> group::Group {
    let mut request_group = group::Group::new(x, 0, (parent_w - x) / 2, parent_h, None);
    request_group.set_frame(FrameType::FlatBox);

    let mut request_flex = Flex::default_fill().column();
    let mut url_bar = Flex::default_fill().row();
    let mut choice = menu::Choice::default();
    choice.add_choice("GET|POST");
    choice.set_value(0);
    choice.set_trigger(CallbackTrigger::Changed);
    choice.set_callback(method_picker_cb);
    url_bar.fixed(&choice, 75);
    let mut url_input = input::Input::default();
    url_input.set_callback(url_input_cb);
    url_input.set_trigger(CallbackTrigger::Changed);
    let mut send_button = Button::default().with_label("Send");
    send_button.set_callback(send_cb);
    url_bar.fixed(&send_button, 75);
    url_bar.end();

    let mut request_tabs = Tabs::new(x, 0, parent_w - x, parent_h, None);
    request_tabs.handle_overflow(group::TabsOverflow::Pulldown);
    request_tabs.set_selection_color(Color::Dark2);
    // TODO: add a dropdown to choose the body type. This is useful for formatting the body later.
    let request_body_tab = Flex::default_fill().with_label("Body\t");
    let mut ed = text::TextEditor::default().with_id("ed");
    ed.set_buffer(request_buffer);
    ed.set_linenumber_width(40);
    ed.set_text_font(Font::Courier);
    request_body_tab.end();
    request_tabs.add(&request_headers_tab);
    let request_auth_tab = Flex::default_fill().with_label("Auth\t");
    request_auth_tab.end();
    let request_query_tab = Flex::default_fill().with_label("Query\t");
    request_query_tab.end();
    request_tabs.end();
    request_tabs.auto_layout();

    request_flex.fixed(&url_bar, 30);
    request_flex.end();

    request_group
}

fn build_response_section(
    x: i32,
    parent_w: i32,
    parent_h: i32,
    response_buffer: text::TextBuffer,
    response_headers: Flex,
    status_code: frame::Frame
) -> group::Group {
    let mut response_group = group::Group::new(x, 0, parent_w / 2, parent_h, None);
    response_group.set_frame(FrameType::FlatBox);
    let mut response_flex = Flex::default_fill().column();
    let mut response_status_flex = Flex::default_fill().row();
    response_status_flex.add(&status_code);
    frame::Frame::default().with_label("0 ms");
    frame::Frame::default().with_label("0 B");
    response_status_flex.end();
    response_flex.fixed(&response_status_flex, 30);

    let mut response_tabs = Tabs::new(x, 0, parent_w, parent_h, None);
    response_tabs.handle_overflow(group::TabsOverflow::Pulldown);
    response_tabs.set_selection_color(Color::Dark2);
    let response_json_tab = Flex::default_fill().with_label("Preview\t");
    let mut ed = text::TextDisplay::default().with_id("ed");
    ed.set_buffer(response_buffer);
    ed.set_linenumber_width(40);
    ed.set_text_font(Font::Courier);
    response_json_tab.end();
    let mut response_headers_tab = Flex::default_fill().with_label("Headers\t");
    response_headers_tab.add(&response_headers);
    response_headers_tab.end();
    response_tabs.end();
    response_tabs.auto_layout();

    response_flex.end();

    response_group
}

fn main() {
    let app = app::App::default();
    let widget_theme = WidgetTheme::new(ThemeType::Metro);
    widget_theme.apply();
    let mut main_win = Window::default().with_size(1000, 750).with_label("Wyduz");
    main_win.make_resizable(true);

    let mut request_buffer = text::TextBuffer::default();
    request_buffer.set_tab_distance(4);

    let mut response_buffer = text::TextBuffer::default();
    response_buffer.set_tab_distance(4);

    let request_headers_tab = build_request_headers_tab();
    let mut response_headers = Flex::default().column();
    response_headers.set_margin(10);
    response_headers.end();

    let status_code = frame::Frame::default();

    let state = State::new(
        request_buffer.clone(),
        response_buffer.clone(),
        request_headers_tab.clone(),
        response_headers.clone(),
        status_code.clone(),
    );
    app::GlobalState::new(state);

    let full_window_tile = build_tile();

    let mut files = frame::Frame::new(0, 0, 200, main_win.h(), "N/A");
    files.set_frame(FrameType::DownBox);
    files.set_color(Color::by_index(9));
    files.set_label_size(36);
    files.set_align(Align::Clip);

    // Request section
    // ---------------
    let request_group = build_request_section(
        files.w(),
        main_win.w(),
        main_win.h(),
        request_buffer,
        request_headers_tab,
    );
    request_group.end();

    // Response section
    // ---------------
    let response_group = build_response_section(
        request_group.x() + request_group.w(),
        main_win.w() - files.w(),
        main_win.h(),
        response_buffer,
        response_headers,
        status_code
    );
    response_group.end();

    full_window_tile.end();
    main_win.end();
    main_win.show();
    app.run().unwrap();
}
