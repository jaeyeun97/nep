extern crate dbus;

use dbus::tree::Factory;
use dbus::{BusType, Connection, NameFlag};
use std::sync::Arc;

fn main() {
    let c = Connection::get_private(BusType::Session).unwrap();
    c.register_name("org.nep", NameFlag::ReplaceExisting as u32)
        .unwrap();

    let f = Factory::new_fn::<()>();

    let tree = f.tree(()).add(
        f.object_path("/hello", ())
            .introspectable()
            .add(
                f.interface("org.nep", ())
                    .add_m(f.method("Hello", (), move |m| {
                        println!("Hello!");
                        Ok(vec![])
                    })),
            ),
    );

    tree.set_registered(&c, true).unwrap();
    c.add_handler(tree);

    loop {
        c.incoming(1000).next();
    }
}
