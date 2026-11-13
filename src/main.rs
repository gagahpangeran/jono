mod css;
mod dom;
mod html;
mod style;

fn main() {
    let html = "
<html>
    <body>
        <div>Hello World!</div>
    </body>
</html>";

    let result = html::parse(html.to_string());
    print!("{:#?}", result);
}
