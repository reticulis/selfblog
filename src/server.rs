use anyhow::Result;
use rocket::{Build, Rocket, routes};
use rocket::fs::NamedFile;
use rocket::fs::{FileServer, relative};
use std::path::{PathBuf, Path};

#[rocket::get("/<path..>")]
async fn main_blog(path: PathBuf) -> Option<NamedFile> {
    let mut path = Path::new(relative!("blog")).join(path);
    if path.is_dir() {
        path.push("index.html");
    }

    NamedFile::open(path).await.ok()
}

pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![main_blog])
        .mount("/", FileServer::from(relative!("blog")))
}