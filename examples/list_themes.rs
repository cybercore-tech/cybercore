fn main() {
    let schema = cybercore::schema::load();
    let names: Vec<_> = schema.theme_names().collect();
    println!("active: {}", schema.active);
    println!("count: {}", names.len());
    for name in names {
        let p = schema.theme(name).unwrap();
        println!("{:<28} bg={} acid_green={}", name, p.bg, p.acid_green);
    }
}
