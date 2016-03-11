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
    let window: PistonWindow = WindowSettings::new("Thor", [1024, 768])
                                   .exit_on_esc(true)
                                   .fullscreen(true)
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
    // let keys_first_row = ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"];
    // let keys_second_row = ["A", "S", "D", "F", "G", "H", "J", "K", "L"];
    // let keys_third_row = ["Z", "X", "C", "V", "B", "N", "M"];


    // Unique ID's for each widget.
    widget_ids!{
        MASTER,
        COL,
        LABEL,
        SUBMIT_BUTTON,
        TEXTBOX,
        BACKSPACE,
        INSTRUCTIONS_LABEL,
        BUTTON_Q,
        BUTTON_W,
        BUTTON_E,
        BUTTON_R,
        BUTTON_T,
        BUTTON_Y,
        BUTTON_U,
        BUTTON_I,
        BUTTON_O,
        BUTTON_P,
        BUTTON_A,
        BUTTON_S,
        BUTTON_D,
        BUTTON_F,
        BUTTON_G,
        BUTTON_H,
        BUTTON_J,
        BUTTON_K,
        BUTTON_L,
        BUTTON_Z,
        BUTTON_X,
        BUTTON_C,
        BUTTON_V,
        BUTTON_B,
        BUTTON_N,
        BUTTON_M,
        BUTTON_SPACE
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

    Text::new("Type in your message and press Enter to send to \
                            everyone.")
        .color(color::BLUE)
        .padded_w_of(COL, PAD)
        .mid_top_of(COL)
        .align_text_middle()
        .line_spacing(2.5)
        .set(INSTRUCTIONS_LABEL, ui);

    Button::new()
        .w_h(200.0, 44.0)
        .bottom_left_of(MASTER)
        .rgb(0.4, 0.75, 0.6)
        .label("Backspace")
        .react(|| {
            let mut n = input.clone();
            n.pop();
            *input = n;
        })
        .set(BACKSPACE, ui);

    Button::new()
        .w_h(200.0, 44.0)
        .bottom_right_of(MASTER)
        .rgb(0.4, 0.75, 0.6)
        .label("Submit")
        .react(|| {
            *submitted = true;
        })
        .set(SUBMIT_BUTTON, ui);

    TextBox::new(input)
        .w_h(300.0, 50.0)
        .font_size(20)
        .mid_bottom_of(COL)
        .rgb(0.4, 0.75, 0.6)
        .react(|_string: &mut String| {
            *submitted = true;
        })
        .set(TEXTBOX, ui);

    Button::new()
        .w_h(200.0, 44.0)
        .bottom_right_of(MASTER)
        .rgb(0.4, 0.75, 0.6)
        .label("Submit")
        .react(|| {
            *submitted = true;
        })
        .set(SUBMIT_BUTTON, ui);

    let button_size = 44.0;
    let button_padding = 5.0;

    // NOTE: This is very much a prototype. Please don't judge.

    Button::new()
        .w_h(button_size, button_size)
        .mid_left_of(MASTER)
        .rgb(0.4, 0.75, 0.6)
        .label("Q")
        .react(|| {
            let mut n = input.clone();
            n.push('Q');
            *input = n;
        })
        .set(BUTTON_Q, ui);

    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_Q, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("W")
        .react(|| {
            let mut n = input.clone();
            n.push('W');
            *input = n;
        })
        .set(BUTTON_W, ui);

    // E.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_W, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("E")
        .react(|| {
            let mut n = input.clone();
            n.push('E');
            *input = n;
        })
        .set(BUTTON_E, ui);

    // R.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_E, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("R")
        .react(|| {
            let mut n = input.clone();
            n.push('R');
            *input = n;
        })
        .set(BUTTON_R, ui);

    // T.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_R, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("T")
        .react(|| {
            let mut n = input.clone();
            n.push('T');
            *input = n;
        })
        .set(BUTTON_T, ui);

    // Y.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_T, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("Y")
        .react(|| {
            let mut n = input.clone();
            n.push('Y');
            *input = n;
        })
        .set(BUTTON_Y, ui);

    // U.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_Y, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("U")
        .react(|| {
            let mut n = input.clone();
            n.push('U');
            *input = n;
        })
        .set(BUTTON_U, ui);

    // I.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_U, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("I")
        .react(|| {
            let mut n = input.clone();
            n.push('I');
            *input = n;
        })
        .set(BUTTON_I, ui);

    // O.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_I, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("O")
        .react(|| {
            let mut n = input.clone();
            n.push('O');
            *input = n;
        })
        .set(BUTTON_O, ui);

    // P.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_O, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("P")
        .react(|| {
            let mut n = input.clone();
            n.push('P');
            *input = n;
        })
        .set(BUTTON_P, ui);

    // A.
    Button::new()
        .w_h(button_size, button_size)
        .down_from(BUTTON_Q, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("A")
        .react(|| {
            let mut n = input.clone();
            n.push('A');
            *input = n;
        })
        .set(BUTTON_A, ui);

    // S.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_A, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("S")
        .react(|| {
            let mut n = input.clone();
            n.push('S');
            *input = n;
        })
        .set(BUTTON_S, ui);

    // D.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_S, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("D")
        .react(|| {
            let mut n = input.clone();
            n.push('D');
            *input = n;
        })
        .set(BUTTON_D, ui);

    // F.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_D, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("F")
        .react(|| {
            let mut n = input.clone();
            n.push('F');
            *input = n;
        })
        .set(BUTTON_F, ui);

    // G.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_F, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("G")
        .react(|| {
            let mut n = input.clone();
            n.push('G');
            *input = n;
        })
        .set(BUTTON_G, ui);

    // H.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_G, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("H")
        .react(|| {
            let mut n = input.clone();
            n.push('H');
            *input = n;
        })
        .set(BUTTON_H, ui);

    // J.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_H, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("J")
        .react(|| {
            let mut n = input.clone();
            n.push('J');
            *input = n;
        })
        .set(BUTTON_J, ui);

    // K.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_J, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("K")
        .react(|| {
            let mut n = input.clone();
            n.push('K');
            *input = n;
        })
        .set(BUTTON_K, ui);

    // L.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_K, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("L")
        .react(|| {
            let mut n = input.clone();
            n.push('L');
            *input = n;
        })
        .set(BUTTON_L, ui);

    // Z.
    Button::new()
        .w_h(button_size, button_size)
        .down_from(BUTTON_A, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("Z")
        .react(|| {
            let mut n = input.clone();
            n.push('Z');
            *input = n;
        })
        .set(BUTTON_Z, ui);

    // X.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_Z, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("X")
        .react(|| {
            let mut n = input.clone();
            n.push('X');
            *input = n;
        })
        .set(BUTTON_X, ui);

    // C.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_X, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("C")
        .react(|| {
            let mut n = input.clone();
            n.push('C');
            *input = n;
        })
        .set(BUTTON_C, ui);

    // V.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_C, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("V")
        .react(|| {
            let mut n = input.clone();
            n.push('V');
            *input = n;
        })
        .set(BUTTON_V, ui);

    // B.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_V, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("V")
        .react(|| {
            let mut n = input.clone();
            n.push('B');
            *input = n;
        })
        .set(BUTTON_B, ui);

    // N.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_B, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("N")
        .react(|| {
            let mut n = input.clone();
            n.push('N');
            *input = n;
        })
        .set(BUTTON_N, ui);

    // M.
    Button::new()
        .w_h(button_size, button_size)
        .right_from(BUTTON_N, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("M")
        .react(|| {
            let mut n = input.clone();
            n.push('M');
            *input = n;
        })
        .set(BUTTON_M, ui);

    // Space.
    Button::new()
        .w_h(200.0, button_size)
        .down_from(BUTTON_Z, button_padding)
        .rgb(0.4, 0.75, 0.6)
        .label("Space")
        .react(|| {
            let mut n = input.clone();
            n.push(' ');
            *input = n;
        })
        .set(BUTTON_SPACE, ui);
}
