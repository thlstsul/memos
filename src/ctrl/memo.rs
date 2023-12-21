use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Result,
    routing::get,
    Json, Router,
};
use libsql_client::Client;
use snafu::{ensure, Snafu};

use crate::{
    api::{
        memo::{FindMemo, Memo},
        system::{SystemSetting, SystemSettingKey},
        v1::memo::ListMemoRequest,
        v2::Visibility,
    },
    svc::{memo::MemoService, system::SystemService, user::UserService},
};

use super::auth::AuthSession;

pub fn router() -> Router<Arc<Client>> {
    Router::new().route("/memo", get(list_memos))
}

async fn list_memos(
    session: AuthSession,
    qry: Query<ListMemoRequest>,
    client: State<Arc<Client>>,
) -> Result<Json<Vec<Memo>>> {
    let svc = MemoService::new(&client);
    let req = qry.0;
    let mut find: FindMemo = req.clone().into();
    if let Some(username) = req.creator_username {
        let svc = UserService::new(&client);
        let user = svc.find_user(username).await?;
        find.creator_id = Some(user.id);
    }
    let current_user = session.user;
    if let Some(user) = current_user {
        let mut visibility_list = vec![Visibility::Public, Visibility::Protected];
        if find.creator_id == Some(user.id) {
            visibility_list.push(Visibility::Private);
        }
        find.visibility_list = visibility_list;
    } else {
        ensure!(find.creator_id.is_some(), MissingUser);
        find.visibility_list = vec![Visibility::Public];
    }

    let sys_svc = SystemService::new(&client);
    let memo_display_with_updated_ts = sys_svc
        .find_setting(SystemSettingKey::MemoDisplayWithUpdatedTs)
        .await?;
    if let Some(SystemSetting { value, .. }) = memo_display_with_updated_ts {
        if value == "true" {
            find.order_by_updated_ts = true;
        }
    }

    let memos = svc.list_memos(find).await?;
    Ok(Json(memos))
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Missing user to find memo"), context(suffix(false)))]
    MissingUser,
}
