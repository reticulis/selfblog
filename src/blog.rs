use crate::database::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct Blog {
    posts: Vec<(String, BlogPost)>,
    // db: Database,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlogPost {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) path: String,
}

impl Blog {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        let posts = Self::get_posts(&db)?;

        Ok(Self { posts /*db*/ })
    }

    fn get_posts(db: &Database) -> Result<Vec<(String, BlogPost)>> {
        db.iter()
            .map(|v| {
                let (id, post) = v?;
                let config = bincode::config::standard();
                let id = String::from_utf8_lossy(&id).to_string();
                let (post, _) = bincode::serde::decode_from_slice::<BlogPost, _>(&post, config)?;

                Ok((id, post))
            })
            .collect::<Result<Vec<(String, BlogPost)>>>()
    }

    pub fn take_articles(&self, from: usize, to: usize) -> &[(String, BlogPost)] {
        &self.posts[from..to]
    }

    pub fn size(&self) -> usize {
        self.posts.len()
    }
}
