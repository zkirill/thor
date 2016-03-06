#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate piston_window;
extern crate websocket;

use std::borrow::Cow;
use std::str::from_utf8;
use std::thread;
use std::io::stdin;
use std::sync::mpsc::channel;
use websocket::{Message, Sender, Receiver};
use websocket::message::Type;
use websocket::client::request::Url;
use websocket::Client;
use websocket::stream::WebSocketStream;
use std::sync::mpsc::{Sender as TrueSender, Receiver as TrueReceiver, RecvError as TrueRecvError,
                      TryRecvError};

use conrod::{color, Scalar, Button, Canvas, Circle, Color, Colorable, DropDownList,
             EnvelopeEditor, Frameable, Labelable, NumberDialer, Point, Positionable, Slider,
             Sizeable, Text, TextBox, Theme, Toggle, Widget, WidgetMatrix, XYPad};

use piston_window::{EventLoop, Glyphs, PistonWindow, UpdateEvent, WindowSettings};

// Use `piston_window` for UI backend.
type Backend = (<piston_window::G2d<'static> as conrod::Graphics>::Texture, Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;


fn main() {
    // Reply from Odin.
    let (reply_tx, reply_rx): (TrueSender<String>, TrueReceiver<String>) = channel();

    let url = Url::parse("ws://127.0.0.1:2794").unwrap();

    println!("Connecting to {}", url);

    let request = Client::connect(url).unwrap();

    let response = request.send().unwrap(); // Send the request and retrieve a response

    println!("Validating response...");

    response.validate().unwrap(); // Validate the response

    println!("Successfully connected");

    let (mut sender, mut receiver) = response.begin().split();

    let (tx, rx) = channel();

    let tx_1 = tx.clone();

    let send_loop = thread::spawn(move || {
        loop {
            // Send loop
            let message: Message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    return;
                }
            };
            match message.opcode {
                Type::Close => {
                    let _ = sender.send_message(&message);
                    // If it's a close message, just send it and then return.
                    return;
                }
                _ => (),
            }
            // Send the message
            match sender.send_message(&message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });

    let receive_loop = thread::spawn(move || {
        // Receive loop
        for message in receiver.incoming_messages() {
            let message: Message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
                    let _ = tx_1.send(Message::close());
                    return;
                }
            };
            match message.opcode {
                Type::Close => {
                    // Got a close message, so send a close message and return
                    let _ = tx_1.send(Message::close());
                    return;
                }
                Type::Ping => {
                    match tx_1.send(Message::pong(message.payload)) {
                        // Send a pong in response
                        Ok(()) => (),
                        Err(e) => {
                            println!("Receive Loop: {:?}", e);
                            return;
                        }
                    }
                }
                // Parse message from Odin.
                _ => {
                    reply_tx.send(from_utf8(&*message.payload).unwrap().to_string());
                }
            }
        }
    });

    // Construct the window.
    let window: PistonWindow = WindowSettings::new("Thor", [1080, 720])
                                   .exit_on_esc(true)
                                   .build()
                                   .unwrap();

    // Construct the UI.
    let mut ui = {
        // Get path for "assets".
        let assets = find_folder::Search::KidsThenParents(3, 5)
                         .for_folder("assets")
                         .unwrap();

        // Get path for the font.
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");

        // Use the "default" theme.
        let theme = Theme::default();

        // Glyph cache.
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());

        // Finally, create the UI.
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // Did the user click on the button?
    let mut clicked = false;
    // Did we press Enter in the textbox?
    let mut submitted = false;
    // Textbox input.
    let mut input: String = "".to_string();
    // Reply from Odin.
    let mut reply = String::new();

    // Poll events from the window.
    for event in window.ups(60) {

        // Listen to sockets without blocking.
        let res = reply_rx.try_recv();
        match res {
            Ok(val) => reply = val,
            Err(err) => {
                match err {
                    TryRecvError::Empty => {
                        // Channel is empty. This event fires frequently.
                    }
                    TryRecvError::Disconnected => {
                        // Channel is disconnected.
                    }
                }
            } 
        }
        ui.handle_event(&event);
        event.update(|_| {
            ui.set_widgets(|mut ui| {
                set_widgets(&mut ui,
                            &mut reply,
                            &mut clicked,
                            &mut submitted,
                            &mut input)
            })
        });
        event.draw_2d(|c, g| ui.draw(c, g));
        if clicked == true {
            let message = Message::text("This drink... I like it! ANOTHER!");
            tx.send(message);
            clicked = false;
        }
        if submitted == true {
            let message = Message::text(input);
            tx.send(message);
            // Clear input.
            input = "".to_string();
            // Reset submitted state.
            submitted = false;
        }
    }
}

fn set_widgets(ui: &mut UiCell,
               text: &mut String,
               clicked: &mut bool,
               submitted: &mut bool,
               input: &mut String) {

    // Unique ID's for each widget.
    widget_ids!{
        MASTER,
        COL,
        LABEL,
        BUTTON,
        TEXTBOX,
        INSTRUCTIONS_LABEL,
    }

    // Create the canvas.
    Canvas::new()
        .flow_right(&[(COL, Canvas::new().color(color::WHITE))])
        .set(MASTER, ui);

    const PAD: Scalar = 20.0;

    Text::new(text)
        .color(color::BLUE)
        .padded_w_of(COL, PAD)
        .middle_of(COL)
        .align_text_middle()
        .line_spacing(2.5)
        .set(LABEL, ui);

    Text::new("Enter your message in the green field and press Enter to send to \
                            everyone.")
        .color(color::BLUE)
        .padded_w_of(COL, PAD)
        .mid_top_of(COL)
        .align_text_middle()
        .line_spacing(2.5)
        .set(INSTRUCTIONS_LABEL, ui);

    Button::new()
        .w_h(300.0, 50.0)
        .mid_left_of(MASTER)
        .rgb(0.4, 0.75, 0.6)
        .label("Send a message to Odin")
        .react(|| {
            *clicked = true;
        });

    TextBox::new(input)
        .w_h(300.0, 50.0)
        .font_size(20)
        .mid_bottom_of(COL)
        .rgb(0.4, 0.75, 0.6)
        .react(|_string: &mut String| {
            *submitted = true;
        })
        .set(TEXTBOX, ui);
}
