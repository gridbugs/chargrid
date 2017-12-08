extern crate prototty;
extern crate prototty_elements;
extern crate cgmath;
extern crate ansi_colour;

use std::time::Duration;
use std::thread;
use ansi_colour::colours;
use prototty::*;
use prototty_elements::elements::*;
use prototty_elements::menu::*;

fn main() {
    let mut ctx = Context::new().unwrap();

    let mut menu = Menu::new(vec![
        ("One", 1),
        ("Two", 2),
        ("Three", 3),
        ("Four", 4),
    ], (6, 4));

    menu.selected_info.foreground_colour = colours::RED;

    let mut menu_instance = Border::new(MenuInstance::new(menu).unwrap());

    let choice = loop {
        match ctx.run_menu(&mut menu_instance, |v| &mut v.child).unwrap() {
            MenuChoice::Quit => return,
            MenuChoice::Cancel => break None,
            MenuChoice::Finalise(x) => break Some(x),
        }
    };

    if let Some(x) = choice {
        ctx.render(&Text::one_line(format!("You chose: {}", x))).unwrap();
    } else {
        ctx.render(&Text::one_line("You chose nothing!")).unwrap();
    }

    thread::sleep(Duration::from_millis(1000));
}
