use cel_parser::{parse, ArithmeticOp, Atom, Expression, RelationOp, UnaryOp};
use libsql::Value;
use sql_query_builder::Select;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    api::v1::gen::Direction,
    dao::turso::ToCriteria,
    model::{
        memo::{FindMemo, FindMemoPayload},
        pager::Paginator as _,
    },
};

impl ToCriteria for FindMemo {
    fn to_criteria(self) -> (impl AsRef<str>, impl libsql::params::IntoParams) {
        let FindMemo {
            id,
            uid,
            state,
            creator_id,
            created_ts_after,
            created_ts_before,
            updated_ts_after,
            updated_ts_before,
            content_search,
            visibility_list,
            payload_find,
            exclude_content,
            page_token,
            only_payload,
            order_by_pinned,
            order_by_updated_ts,
            filter,
            sort,
            direction,
            ..
        } = self;

        let mut sql = Select::new()
            .from("memo")
            .left_join("JSON_EACH(memo.payload, '$.tags')");
        let mut params = Vec::new();

        if only_payload {
            sql = sql
                .select("memo.id AS id")
                .select("memo.payload AS payload")
                .select("memo.pinned AS pinned")
                .select("memo.created_ts AS created_ts")
                .select("memo.updated_ts AS updated_ts");
        } else {
            sql = sql
                .select("memo.id AS id")
                .select("memo.uid AS uid")
                .select("memo.creator_id AS creator_id")
                .select("memo.created_ts AS created_ts")
                .select("memo.updated_ts AS updated_ts")
                .select("memo.row_status AS state")
                .select("memo.visibility AS visibility")
                .select("memo.pinned AS pinned")
                .select("memo.payload AS payload");
        };

        if !exclude_content && !only_payload {
            sql = sql.select("memo.content AS content");
        }

        if let Some(id) = id {
            sql = sql.where_and("memo.id = ?");
            params.push(Value::from(id));
        }
        if let Some(uid) = uid {
            sql = sql.where_and("memo.uid = ?");
            params.push(Value::from(uid))
        }
        if let Some(creator_id) = creator_id {
            sql = sql.where_and("memo.creator_id = ?");
            params.push(Value::from(creator_id));
        }
        if let Some(state) = &state {
            sql = sql.where_and("memo.row_status = ?");
            params.push(Value::from(state.to_string()));
        }
        if let Some(created_ts_before) = created_ts_before {
            sql = sql.where_and("memo.created_ts < ?");
            params.push(Value::from(created_ts_before));
        }
        if let Some(created_ts_after) = created_ts_after {
            sql = sql.where_and("memo.created_ts > ?");
            params.push(Value::from(created_ts_after));
        }
        if let Some(updated_ts_before) = updated_ts_before {
            sql = sql.where_and("memo.updated_ts < ?");
            params.push(Value::from(updated_ts_before));
        }
        if let Some(updated_ts_after) = updated_ts_after {
            sql = sql.where_and("memo.updated_ts > ?");
            params.push(Value::from(updated_ts_after));
        }
        for content_search in content_search.iter() {
            sql = sql.where_and("memo.content LIKE ?");
            params.push(Value::from(format!("%{content_search}%")));
        }

        let w;
        if !visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in visibility_list.iter() {
                params.push(Value::from(visibility.as_str_name().to_owned()));
                l.push("?");
            }
            w = format!("memo.visibility in ({})", l.join(", "));
            sql = sql.where_and(w.as_str());
        }

        if let Some(FindMemoPayload {
            raw,
            tags,
            has_link,
            has_task_list,
            has_code,
            has_incomplete_tasks,
        }) = payload_find
        {
            if let Some(raw) = raw {
                sql = sql.where_and("memo.payload = ?");
                params.push(Value::from(raw));
            }
            if let Some(tags) = tags {
                for tag in tags {
                    sql = sql.where_and("JSON_EACH.value = ?");
                    params.push(Value::from(tag));
                }
            }
            if has_link {
                sql = sql.where_and("JSON_EXTRACT(memo.payload, '$.property.has_link') IS TRUE");
            }
            if has_task_list {
                sql =
                    sql.where_and("JSON_EXTRACT(memo.payload, '$.property.has_task_list') IS TRUE");
            }
            if has_code {
                sql = sql.where_and("JSON_EXTRACT(memo.payload, '$.property.has_code') IS TRUE");
            }
            if has_incomplete_tasks {
                sql = sql.where_and(
                    "JSON_EXTRACT(memo.payload, '$.property.has_incomplete_tasks') IS TRUE",
                );
            }
        }

        let (filter_sql, mut filter_params) = convert_ecl_to_sql(&filter);
        if !filter_sql.is_empty() {
            sql = sql.where_and(filter_sql.as_str());
            params.append(&mut filter_params);
        }

        if !only_payload && order_by_pinned {
            sql = sql.order_by("pinned DESC");
        }

        if !only_payload || direction != Direction::Unspecified {
            if order_by_updated_ts {
                sql = sql.order_by(&format!("updated_ts {}", direction.as_str_name()));
            } else {
                sql = sql.order_by(&format!("created_ts {}", direction.as_str_name()));
            }
        }

        if let Some(page_token) = page_token {
            sql = sql
                .limit(&page_token.limit().to_string())
                .offset(&page_token.offset().to_string());
        }

        (sql.as_string(), params)
    }
}

fn convert_ecl_to_sql(filter: &str) -> (String, Vec<Value>) {
    if filter.is_empty() {
        return (String::default(), Vec::default());
    }

    let expr = parse(filter).unwrap();
    if matches!(expr, Expression::Atom(_)) {
        return (String::default(), Vec::default());
    }

    convert_expr_to_sql(expr)
}

fn convert_expr_to_sql(expr: Expression) -> (String, Vec<Value>) {
    match expr {
        Expression::Or(expr1, expr2) => {
            let (sql1, mut param1) = convert_expr_to_sql(*expr1);
            let (sql2, mut param2) = convert_expr_to_sql(*expr2);
            param1.append(&mut param2);
            (format!("({sql1} OR {sql2})"), param1)
        }
        Expression::And(expr1, expr2) => {
            let (sql1, mut param1) = convert_expr_to_sql(*expr1);
            let (sql2, mut param2) = convert_expr_to_sql(*expr2);
            param1.append(&mut param2);
            (format!("({sql1} AND {sql2})"), param1)
        }
        Expression::Relation(expr1, op, expr2) => {
            let (sql1, mut param1) = convert_expr_to_sql(*expr1);
            let (sql2, mut param2) = if sql1.contains("time") {
                if let Expression::Atom(Atom::String(s)) = *expr2 {
                    if let Ok(time) = OffsetDateTime::parse(&s, &Rfc3339) {
                        ("?".to_string(), vec![Value::from(time.unix_timestamp())])
                    } else {
                        ("?".to_string(), vec![Value::from(s.to_string())])
                    }
                } else {
                    convert_expr_to_sql(*expr2)
                }
            } else {
                convert_expr_to_sql(*expr2)
            };
            param1.append(&mut param2);
            let op = match op {
                RelationOp::Equals => "=",
                RelationOp::NotEquals => "<>",
                RelationOp::LessThan => "<",
                RelationOp::LessThanEq => "<=",
                RelationOp::GreaterThan => ">",
                RelationOp::GreaterThanEq => ">=",
                RelationOp::In => "IN",
            };
            (format!("({sql1} {op} {sql2})"), param1)
        }
        Expression::Unary(op, expr) => {
            let (sql, param) = convert_expr_to_sql(*expr);
            let op = match op {
                UnaryOp::Not => "NOT ",
                UnaryOp::Minus => "-",
                _ => "",
            };
            (format!("{op}({sql})"), param)
        }
        Expression::List(expr_list) => {
            let mut sql = String::new();
            let mut param = Vec::new();
            for (i, expr) in expr_list.iter().enumerate() {
                let (s, mut p) = convert_expr_to_sql(expr.clone());
                if i == 0 {
                    sql.push_str(&s);
                } else {
                    sql.push_str(&format!(", {s}"));
                }
                param.append(&mut p);
            }
            (format!("({sql})"), param)
        }
        Expression::Atom(atom) => match atom {
            Atom::Int(int) => ("?".to_string(), vec![Value::from(int)]),
            Atom::UInt(uint) => ("?".to_string(), vec![Value::from(uint as i64)]),
            Atom::Float(float) => ("?".to_string(), vec![Value::from(float)]),
            Atom::String(s) => ("?".to_string(), vec![Value::from(s.to_string())]),
            Atom::Bytes(bytes) => ("?".to_string(), vec![Value::from(bytes.to_vec())]),
            Atom::Bool(bl) => ("?".to_string(), vec![Value::from(if bl { 1 } else { 0 })]),
            Atom::Null => ("NULL".to_string(), Vec::default()),
        },
        Expression::Ident(ident) => {
            let ident = match ident.as_str() {
                "tag" => "JSON_EACH.value",
                "pinned" => "pinned IS TRUE",
                "has_link" => "JSON_EXTRACT(payload, '$.property.has_link') IS TRUE",
                "has_task_list" => "JSON_EXTRACT(payload, '$.property.has_task_list') IS TRUE",
                "has_code" => "JSON_EXTRACT(payload, '$.property.has_code') IS TRUE",
                "has_incomplete_tasks" => {
                    "JSON_EXTRACT(payload, '$.property.has_incomplete_tasks') IS TRUE"
                }
                _ => ident.as_str(),
            };
            (ident.to_string(), Vec::default())
        }
        Expression::Arithmetic(expr1, op, expr2) => {
            let (sql1, mut param1) = convert_expr_to_sql(*expr1);
            let (sql2, mut param2) = convert_expr_to_sql(*expr2);
            param1.append(&mut param2);
            let op = match op {
                ArithmeticOp::Add => " + ",
                ArithmeticOp::Subtract => " - ",
                ArithmeticOp::Multiply => " * ",
                ArithmeticOp::Divide => " / ",
                _ => return (String::default(), Vec::default()),
            };
            (format!("({sql1} {op} {sql2})"), param1)
        }
        Expression::FunctionCall(expr, that, args) => {
            let (sql, mut param1) = convert_expr_to_sql(*expr);
            if "contains" == sql && args.len() == 1 {
                if let (Some(that), Expression::Atom(Atom::String(arg))) = (that, args[0].clone()) {
                    let (that, mut param) = convert_expr_to_sql(*that);
                    param.append(&mut param1);
                    param.push(Value::from(format!("%{arg}%")));
                    (format!("{that} LIKE ?"), param)
                } else {
                    (String::default(), Vec::default())
                }
            } else {
                let mut sql = String::new();
                for (i, arg) in args.iter().enumerate() {
                    let (s, mut p) = convert_expr_to_sql(arg.clone());
                    if i == 0 {
                        sql.push_str(&s);
                    } else {
                        sql.push_str(&format!(", {s}"));
                    }
                    param1.append(&mut p);
                }
                (format!("{sql}({})", sql), param1)
            }
        }
        _ => (String::default(), Vec::default()),
    }
}

#[test]
fn test() {
    let (sql, params) = convert_ecl_to_sql(
        r#"!(tag in ["tag1", "tag2"]) && (content.contains('hello') || pinned) && create_time == "2006-01-02T15:04:05+07:00""#,
    );
    assert_eq!(
        r#"((NOT ((JSON_EACH.value IN (?, ?))) AND (content LIKE ? OR pinned IS TRUE)) AND (create_time = ?))"#,
        sql
    );
    assert_eq!(
        vec![
            Value::from("tag1"),
            Value::from("tag2"),
            Value::from("%hello%"),
            Value::from(1136189045)
        ],
        params
    );
}
