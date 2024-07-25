use async_trait::async_trait;
use std::collections::HashMap;

use libsql::Value;

use crate::{
    dao::resource::{
        CreateResourceError, DeleteResourceError, GetResourceError, ListResourceError,
        RelateResourceError, ResourceRepository, SetResourceError,
    },
    model::{
        gen::ResourceStorageType,
        resource::{FindResource, Resource},
    },
};

use super::{de, Turso};

#[async_trait]
impl ResourceRepository for Turso {
    async fn create_resource(
        &self,
        Resource {
            filename,
            r#type,
            size,
            creator_id,
            blob,
            id,
            created_ts,
            updated_ts,
            memo_id,
            uid,
            reference,
            storage_type,
            payload,
        }: Resource,
    ) -> Result<Option<Resource>, CreateResourceError> {
        let mut fields = vec![
            "uid",
            "filename",
            "type",
            "size",
            "creator_id",
            "storage_type",
        ];
        let mut placeholder = vec!["?", "?", "?", "?", "?", "?"];
        let mut storage_type_str = "";
        if storage_type != ResourceStorageType::Unspecified {
            storage_type_str = storage_type.as_str_name();
        }
        let mut args = vec![
            Value::from(uid),
            Value::from(filename),
            Value::from(r#type),
            Value::from(size as u32),
            Value::from(creator_id),
            Value::from(storage_type_str),
        ];

        if !blob.is_empty() {
            fields.push("blob");
            placeholder.push("?");
            args.push(Value::from(blob));
        }

        if !reference.is_empty() {
            fields.push("reference");
            placeholder.push("?");
            args.push(Value::from(reference));
        }

        fields.push("payload");
        placeholder.push("?");
        args.push(payload.into());

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
            "insert into resource ({}) values ({}) returning id, memo_id, uid, filename, type, size, created_ts, reference",
            fields.join(", "),
            placeholder.join(", ")
        );

        let mut rs = self.query(&insert_sql, args).await?;
        Ok(rs.pop())
    }

    async fn set_resources_memo(
        &self,
        memo_id: i32,
        add_res_ids: Vec<i32>,
        del_res_ids: Vec<i32>,
    ) -> Result<(), SetResourceError> {
        if add_res_ids.is_empty() && del_res_ids.is_empty() {
            return Ok(());
        }

        let transaction = self.transaction().await?;
        if !add_res_ids.is_empty() {
            let mut stmt =
                Self::tx_prepare(&transaction, "update resource set memo_id = ? where id = ?")
                    .await?;
            for add_id in add_res_ids {
                Self::statement_execute(&mut stmt, [memo_id, add_id]).await?;
                stmt.reset();
            }
        }

        if !del_res_ids.is_empty() {
            let mut stmt = Self::tx_prepare(
                &transaction,
                "delete from resource where memo_id = ? and id = ?",
            )
            .await?;
            for del_id in del_res_ids {
                Self::statement_execute(&mut stmt, [memo_id, del_id]).await?;
                stmt.reset();
            }
        }
        Self::commit(transaction).await?;

        Ok(())
    }

    async fn get_resource(&self, id: i32) -> Result<Option<Resource>, GetResourceError> {
        let sql = "select * from resource where id = ?";
        let mut rs = self.query(sql, [id]).await?;
        Ok(rs.pop())
    }

    async fn list_resources(
        &self,
        FindResource {
            id,
            creator_id,
            filename,
            memo_id,
            limit,
            offset,
            has_relate_memo,
            uid,
            storage_type,
            get_blob,
        }: FindResource,
    ) -> Result<Vec<Resource>, ListResourceError> {
        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = id {
            wheres.push("id = ?");
            args.push(Value::from(id));
        }
        if let Some(uid) = uid {
            wheres.push("uid = ?");
            args.push(Value::from(uid));
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

        if let Some(storage_type) = storage_type {
            wheres.push("storage_type = ?");
            args.push(Value::from(storage_type.as_str_name()));
        }

        if has_relate_memo {
            wheres.push("memo_id IS NOT NULL");
        }

        let mut fields =
            "id, uid, filename, reference, type, size, created_ts, memo_id".to_string();
        if get_blob {
            fields = format!("{fields}, blob as content");
        }

        let mut sql = format!(
            "select {} from resource where {}",
            fields,
            wheres.join(" AND ")
        );

        if let Some(limit) = limit {
            sql = format!("{sql} LIMIT {limit}");
            if let Some(offset) = offset {
                sql = format!("{sql} OFFSET {offset}");
            }
        }

        Ok(self.query(&sql, args).await?)
    }

    async fn delete_resource(&self, id: i32, creator_id: i32) -> Result<(), DeleteResourceError> {
        let sql = "delete from resource where id = ? and creator_id = ?";
        self.execute(sql, [id, creator_id]).await?;
        Ok(())
    }

    async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<Resource>>, RelateResourceError> {
        if memo_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut rtn = HashMap::new();
        let mut stmt = self
            .prepare("select id, uid, filename, reference, type, size, created_ts, memo_id from resource where memo_id = ?").await?;
        for memo_id in memo_ids {
            let rows = Self::statement_query(&mut stmt, [memo_id]).await?;
            let res = de(rows).await?;
            rtn.insert(memo_id, res);
            stmt.reset();
        }

        Ok(rtn)
    }
}
