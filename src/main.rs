use std::{fs::File, io::BufWriter};

mod css;
mod dom;
mod html;
mod layout;
mod painting;
mod style;

fn main() {
    let html = "
<div></div>
";
    let css = "
div {
    width: 400px;
    height: 400px;
    background: #ff0000;
    display: block;
}
";

    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;

    let root_node = html::parse(html.to_string());
    let stylesheet = css::parse(css.to_string());
    let style_root = style::style_tree(&root_node, &stylesheet);
    let layout_root = layout::layout_tree(&style_root, viewport);

    let filename = "output.png";
    let mut file = BufWriter::new(File::create(&filename).unwrap());

    let canvas = painting::paint(&layout_root, viewport.content);
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
        let color = canvas.pixels[(y * w + x) as usize];
        image::Rgba([color.r, color.g, color.b, color.a])
    });

    let ok = image::DynamicImage::ImageRgba8(img)
        .write_to(&mut file, image::ImageFormat::Png)
        .is_ok();

    if ok {
        print!("Saved output as {}", filename);
    } else {
        print!("Error saving output as {}", filename);
    }
}
