use rocket::response::content::Xml;

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;


#[get("/")]
fn index() -> Xml<String> {
    let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
            <soap:Body>
                <Response>
                    <Message>Hello, World!</Message>
                </Response>
            </soap:Body>
        </soap:Envelope>
    "#;

    Xml(response.to_string())
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}