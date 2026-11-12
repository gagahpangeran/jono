mod dom;

fn main() {
    let text = dom::text("Hello World".to_string());
    let div = dom::elem("div".to_string(), dom::AttrMap::new(), vec![text]);
    print!("{:#?}", div);
}
