/* Custom data types :
 - `struct`: define a structure
 - `enum`: define an enumeration

 Constants can be created by:
 `const` and `static` keywords
*/

/*
Structures

There are 3 types of structures:
- tuple structs, is tuples
- The classic C structs
- Unit structs, which are field-less, useful for generic
 */

// An attribute to hide warnings for unused code
#![allow(dead_code)]

// define structure Person as debug
#[derive(Debug)]
struct Person {
    name: String,
    age: u8,
}

// Define a unit struct
struct Unit;

// Define a tuple struct
struct Pair(i32, f32);

// Define a struct with two fields
struct Point {
    x: f32,
    y: f32,
}

// Reuse struct Point in this struct
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

fn main() {
    // Use struct Person
    let name = String::from("Peter");
    let age = 27;
    let peter = Person { name, age };
    println!("Person {:?}", peter);

    // use Point
    let point: Point = Point { x: 10.3, y: 0.4 };
    println!("point coordinate: ({}, {})", point.x, point.y);

    // Use rectangle
    // the second point is point.y
    let bottom_right: Point = Point { x: 5.2, ..point };

    println!("second point : ({}, {})", bottom_right.x, bottom_right.y);

    // Destruct the point using the let binding
    let Point { x: left_edge, y: top_edge } = point;

    let _rectangle = Rectangle {
        top_left: Point { x: left_edge, y: top_edge },
        bottom_right: bottom_right,
    };

    // use unit struct
    let _unit = Unit;

    // Use pair struct
    let pair = Pair(1, 0.1);
    println!("pair contains {:?} and {:?}", pair.0, pair.1);

    // Destructure a tuple struct
    let Pair(integer, decimal) = pair;
    println!("pair contains {:?} and {:?}", integer, decimal);
}