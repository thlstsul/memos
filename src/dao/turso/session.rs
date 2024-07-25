use crate::{
    dao::session::{
        CreateSessionError, DeleteExpiredSessionError, DeleteSessionError, GetSessionError,
        MigrateSessionTableError, SessionRepository,
    },
    model::session::Session,
};
use async_trait::async_trait;
use libsql::params;
use time::OffsetDateTime;

use super::Turso;

const TABLE_NAME: &str = "sessions";

#[async_trait]
impl SessionRepository for Turso {
    async fn create_session(&self, session: Session) -> Result<(), CreateSessionError> {
        let sql = format!(
            r#"
            insert into {}
              (id, data, expiry_date) values (?, ?, ?)
            on conflict(id) do update set
              data = excluded.data,
              expiry_date = excluded.expiry_date
            "#,
            TABLE_NAME
        );

        self.execute(&sql, params![session.id, session.data, session.expiry_date])
            .await?;
        Ok(())
    }

    async fn get_session(&self, session_id: String) -> Result<Option<Session>, GetSessionError> {
        let sql = format!(
            r#"
            select id, data, expiry_date from {}
            where id = ? and expiry_date > ?
            "#,
            TABLE_NAME
        );

        let mut rs = self
            .query(
                &sql,
                params![session_id, OffsetDateTime::now_utc().unix_timestamp()],
            )
            .await?;
        Ok(rs.pop())
    }

    async fn delete_session(&self, session_id: String) -> Result<(), DeleteSessionError> {
        let sql = format!(
            r#"
            delete from {} where id = ?
            "#,
            TABLE_NAME
        );
        self.execute(&sql, [session_id]).await?;
        Ok(())
    }

    async fn delete_expired_session(&self) -> Result<(), DeleteExpiredSessionError> {
        let sql = format!(
            r#"
            delete from {}
            where expiry_date < datetime('now', 'utc')
            "#,
            TABLE_NAME
        );
        self.execute(&sql, ()).await?;
        Ok(())
    }

    async fn migrate_session_table(&self) -> Result<(), MigrateSessionTableError> {
        let sql = format!(
            r#"
            create table if not exists {}
            (
                id text primary key not null,
                data blob not null,
                expiry_date integer not null
            )
            "#,
            TABLE_NAME
        );
        self.execute(&sql, ()).await?;
        Ok(())
    }
}
