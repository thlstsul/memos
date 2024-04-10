use std::collections::HashMap;

use libsql::Value;
use snafu::{ResultExt, Snafu};

use crate::{
    api::{
        resource::{FindResource, WholeResource},
        v2::Resource,
    },
    state::AppState,
};

use super::{de, Dao};

pub struct ResourceDao {
    pub state: AppState,
}

impl Dao for ResourceDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl ResourceDao {
    pub async fn create_resource(
        &self,
        WholeResource {
            filename,
            r#type,
            size,
            creator_id,
            blob,
            external_link,
            internal_path,
            id,
            resource_name,
            created_ts,
            updated_ts,
            memo_id,
        }: WholeResource,
    ) -> Result<Option<Resource>, super::Error> {
        let mut fields = vec!["resource_name", "filename", "type", "size", "creator_id"];
        let mut placeholder = vec!["?", "?", "?", "?", "?"];
        let mut args = vec![
            Value::from(resource_name),
            Value::from(filename),
            Value::from(r#type),
            Value::from(size as u32),
            Value::from(creator_id),
        ];

        if !blob.is_empty() {
            fields.push("blob");
            placeholder.push("?");
            args.push(Value::from(blob));
        }

        if !external_link.is_empty() {
            fields.push("external_link");
            placeholder.push("?");
            args.push(Value::from(external_link));
        }

        if !internal_path.is_empty() {
            fields.push("internal_path");
            placeholder.push("?");
            args.push(Value::from(internal_path));
        }

        if id > 0 {
            fields.push("id");
            placeholder.push("?");
            args.push(Value::from(id));
        }

        if created_ts > 0 {
            fields.push("created_ts");
            placeholder.push("?");
            args.push(Value::from(created_ts));
        }

        if updated_ts > 0 {
            fields.push("updated_ts");
            placeholder.push("?");
            args.push(Value::from(updated_ts));
        }

        if let Some(memo_id) = memo_id {
            fields.push("memo_id");
            placeholder.push("?");
            args.push(Value::from(memo_id));
        }

        let insert_sql = format!(
            "insert into resource ({}) values ({}) returning id, memo_id, resource_name as name, filename, type, size, created_ts as create_time, external_link",
            fields.join(", "),
            placeholder.join(", ")
        );

        let mut rs = self.query(&insert_sql, args).await?;
        Ok(rs.pop())
    }

    pub async fn set_memo_resources(
        &self,
        memo_id: i32,
        add_res_ids: Vec<i32>,
        del_res_ids: Vec<i32>,
    ) -> Result<(), libsql::Error> {
        if add_res_ids.is_empty() && del_res_ids.is_empty() {
            return Ok(());
        }

        let transaction = self.get_state().transaction().await?;
        if !add_res_ids.is_empty() {
            let mut stmt = transaction
                .prepare("update resource set memo_id = ? where id = ?")
                .await?;
            for add_id in add_res_ids {
                stmt.execute([memo_id, add_id]).await?;
                stmt.reset();
            }
        }

        if !del_res_ids.is_empty() {
            let mut stmt = transaction
                .prepare("delete from resource where memo_id = ? and id = ?")
                .await?;
            for del_id in del_res_ids {
                stmt.execute([memo_id, del_id]).await?;
                stmt.reset();
            }
        }
        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_resource(&self, id: i32) -> Result<Option<WholeResource>, super::Error> {
        let sql = "select * from resource where id = ?";
        let mut rs = self.query(sql, [id]).await?;
        Ok(rs.pop())
    }

    pub async fn list_resources(
        &self,
        FindResource {
            id,
            name,
            creator_id,
            filename,
            memo_id,
            limit,
            offset,
            has_relate_memo,
        }: FindResource,
    ) -> Result<Vec<Resource>, super::Error> {
        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = id {
            wheres.push("id = ?");
            args.push(Value::from(id));
        }
        if let Some(name) = name {
            wheres.push("name = ?");
            args.push(Value::from(name));
        }

        if let Some(creator_id) = creator_id {
            wheres.push("creator_id = ?");
            args.push(Value::from(creator_id));
        }

        if let Some(filename) = filename {
            wheres.push("filename = ?");
            args.push(Value::from(filename));
        }

        if let Some(memo_id) = memo_id {
            wheres.push("memo_id = ?");
            args.push(Value::from(memo_id));
        }

        if has_relate_memo {
            wheres.push("memo_id IS NOT NULL");
        }

        let mut sql = format!("select id, resource_name as name, filename, external_link, type, size, created_ts as create_time, memo_id from resource where {}", wheres.join(" AND "));

        if let Some(limit) = limit {
            sql = format!("{sql} LIMIT {limit}");
            if let Some(offset) = offset {
                sql = format!("{sql} OFFSET {offset}");
            }
        }

        self.query(&sql, args).await
    }

    pub async fn delete_resource(&self, id: i32, creator_id: i32) -> Result<(), super::Error> {
        let sql = "delete from resource where id = ? and creator_id = ?";
        self.execute(sql, [id, creator_id]).await?;
        Ok(())
    }

    pub async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<Resource>>, Error> {
        if memo_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut rtn = HashMap::new();
        let mut stmt = self
            .get_state()
            .prepare("select id, resource_name as name, filename, external_link, type, size, created_ts as create_time, memo_id from resource where memo_id = ?")
            .await
            .context(PrepareStatement)?;
        for memo_id in memo_ids {
            let rows = stmt.query([memo_id]).await.context(Execute)?;
            let res = de(rows).await.context(Deserialize)?;
            rtn.insert(memo_id, res);
        }

        Ok(rtn)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Execute { source: libsql::Error },
    #[snafu(
        display("Failed to prepare statement: {source}"),
        context(suffix(false))
    )]
    PrepareStatement { source: libsql::Error },
    #[snafu(display("Deserialize failed: {source}"), context(suffix(false)))]
    Deserialize { source: super::Error },
}
