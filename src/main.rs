mod relays;

use relays::{KasaPlug};

fn main() {
    let mut plug = KasaPlug::new("192.168.0.109".parse().unwrap(),
                                 "LampLight".parse().unwrap(),
                                 "office".parse().unwrap());

    println!("{:?}", plug.meta());

}
