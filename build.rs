use std::{fs, process::Command};

use prost_build::Config;
use proto_builder_trait::prost::BuilderAttributes;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/*");

    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    Config::new()
        .out_dir("src/pb")
        .with_serde(
            &[
                "memos.api.v2.User",
                "memos.api.v2.Memo",
                "memos.api.v2.Resource",
                "memos.api.v2.Tag",
                "memos.api.v2.Activity",
                "memos.api.v2.ActivityPayload",
                "memos.api.v2.ActivityMemoCommentPayload",
                "memos.api.v2.ActivityVersionUpdatePayload",
                "memos.api.v2.Inbox",
            ],
            true,
            true,
            None,
        )
        .with_field_attributes(
            &[
                "memos.api.v2.Activity.create_time",
                "memos.api.v2.Inbox.create_time",
            ],
            &[r#"#[serde(with = "crate::pb::time_serde", rename = "create_ts")]"#],
        )
        .with_field_attributes(
            &[
                "memos.api.v2.Resource.r#type",
                "memos.api.v2.Activity.r#type",
            ],
            &[r#"#[serde(rename = "type")]"#],
        )
        .field_attribute(
            "memos.api.v2.Memo.row_status",
            r#"#[serde(with = "crate::pb::status_serde")]"#,
        )
        .field_attribute(
            "memos.api.v2.Resource.created_ts",
            r#"#[serde(with = "crate::pb::time_serde")]"#,
        )
        .field_attribute("memos.api.v2.User.name", r#"#[serde(rename = "username")]"#)
        .field_attribute(
            "memos.api.v2.User.row_status",
            r#"#[serde(with = "crate::pb::status_serde", rename(serialize = "rowStatus"))]"#,
        )
        .field_attribute(
            "memos.api.v2.User.role",
            r#"#[serde(with = "crate::pb::role_serde")]"#,
        )
        .field_attribute(
            "memos.api.v2.User.create_time", 
            r#"#[serde(with = "crate::pb::time_serde", rename(serialize = "createdTs", deserialize = "create_ts"))]"#
        )
        .field_attribute(
            "memos.api.v2.User.update_time",
            r#"#[serde(with = "crate::pb::time_serde", rename(serialize = "updatedTs", deserialize = "update_ts"))]"#,
        )
        .field_attribute(
            "memos.api.v2.User.avatar_url",
            r#"#[serde(rename(serialize = "avatarUrl"))]"#,
        )
        .compile_protos(
            &[
                "proto/api/v2/user_service.proto",
                "proto/api/v2/system_service.proto",
                "proto/api/v2/memo_service.proto",
                "proto/api/v2/resource_service.proto",
                "proto/api/v2/tag_service.proto",
                "proto/api/v2/activity_service.proto",
                "proto/api/v2/inbox_service.proto",
            ],
            // https://github.com/googleapis/googleapis.git
            &["proto", "googleapis"],
        )
        .unwrap();

    fs::remove_file("src/pb/google.api.rs").unwrap();
    fs::rename("src/pb/memos.api.v2.rs", "src/pb/memos_api_v2.rs").unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
}
