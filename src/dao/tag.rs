use libsql::params;

use crate::{api::v2::Tag, state::AppState};

use super::{Dao, Error};

pub struct TagDao {
    pub state: AppState,
}

impl Dao for TagDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl TagDao {
    pub async fn list_tags(&self, creator_id: i32) -> Result<Vec<Tag>, Error> {
        let sql = "select user.username as creator, tag.name from user, tag where user.id = ? and user.id = tag.creator_id";
        self.query(sql, [creator_id]).await
    }

    pub async fn delete_tag(&self, name: &str, creator_id: i32) -> Result<(), Error> {
        let sql = "delete from tag where name = ? and creator_id = ?";
        self.execute(sql, params![name, creator_id]).await?;
        Ok(())
    }

    pub async fn upsert_tag(&self, name: &str, creator_id: i32) -> Result<(), Error> {
        let sql = "
            INSERT INTO tag (
                name, creator_id
            )
            VALUES (?, ?)
            ON CONFLICT(name, creator_id) DO UPDATE 
            SET
                name = EXCLUDED.name";
        self.execute(sql, params![name, creator_id]).await?;
        Ok(())
    }
}
