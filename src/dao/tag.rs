use libsql_client::{Statement, Value};

use crate::{api::v2::Tag, state::AppState};

use super::{Dao, Error};

#[derive(Debug)]
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
        let stmt = Statement::with_args("select user.username as creator, tag.name from user, tag where user.id = ? and user.id = tag.creator_id", &[creator_id]);
        self.query(stmt).await
    }

    pub async fn delete_tag(&self, name: String, creator_id: i32) -> Result<(), Error> {
        let stmt = Statement::with_args(
            "delete from tag where name = ? and creator_id = ?",
            &[Value::from(name), Value::from(creator_id)],
        );
        self.execute(stmt).await?;
        Ok(())
    }

    pub async fn upsert_tag(&self, name: String, creator_id: i32) -> Result<(), Error> {
        let stmt = Statement::with_args(
            "
            INSERT INTO tag (
                name, creator_id
            )
            VALUES (?, ?)
            ON CONFLICT(name, creator_id) DO UPDATE 
            SET
                name = EXCLUDED.name",
            &[Value::from(name), Value::from(creator_id)],
        );
        self.execute(stmt).await?;
        Ok(())
    }
}
