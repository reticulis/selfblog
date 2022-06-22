use rocket::fs::NamedFile;
use rocket::fs::{relative, FileServer};
use rocket::{routes, Build, Rocket};
use std::path::{Path, PathBuf};

#[rocket::get("/<path..>")]
async fn blog(path: PathBuf) -> Option<NamedFile> {
    let mut path = Path::new(relative!("blog")).join(path);
    if path.is_dir() {
        path.push("index.html");
    }

    NamedFile::open(path).await.ok()
}

pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![blog])
        .mount("/", FileServer::from(relative!("blog")))
}
