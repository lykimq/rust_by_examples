// enum allows the creation of a type which may be one of a few
// different variants

enum WebEvent {
    PageLoad,
    PageUnload,
    KeyPress(char),
    Paste(String),
    Click {
        x: i64,
        y: i64,
    },
}

// A function which takes a `WebEvent` enum as an argument
// and return nothing
fn inspect(event: WebEvent) {
    match event {
        WebEvent::PageLoad => println!("page loaded!"),
        WebEvent::PageUnload => println!("page unloaded!"),
        // Destructure `c` from inside the `enum`
        WebEvent::KeyPress(c) => println!("pressed `{}`.", c),
        WebEvent::Paste(s) => println!("pasted \"{}\".", s),
        // Destructure `Click` into `x` and `y`
        WebEvent::Click { x, y } => {
            println!("clicked at x = {}, y = {}", x, y);
        }
    }
}

fn main() {
    let pressed = WebEvent::KeyPress('x');
    // `to_owned()` creates an owned `String` from a string slice
    let pasted = WebEvent::Paste("my text".to_owned());
    let click = WebEvent::Click { x: 20, y: 80 };
    let load = WebEvent::PageLoad;
    let unloaded = WebEvent::PageUnload;

    // call inspect with each event
    inspect(pressed);
    inspect(pasted);
    inspect(click);
    inspect(load);
    inspect(unloaded);
}