extern crate embed_resource;

fn main() {
    embed_resource::compile("assets/tinitime.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
