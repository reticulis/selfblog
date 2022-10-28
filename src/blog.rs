use crate::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Blog {
    posts: Vec<(String, BlogPost)>,
    _db: Arc<Database>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlogPost {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) path: String,
}

impl Blog {
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let posts = Self::get_posts(&db)?;

        Ok(Self { posts, _db: db })
    }

    fn get_posts(db: &Arc<Database>) -> Result<Vec<(String, BlogPost)>> {
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
