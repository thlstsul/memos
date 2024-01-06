use std::{fs, process::Command};

use proto_builder_trait::tonic::BuilderAttributes;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/*");

    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::configure()
        .out_dir("src/api/v2")
        .with_serde(
            &[
                "memos.api.v2.Memo",
                "memos.api.v2.Resource",
                "memos.api.v2.Tag",
                "memos.api.v2.Activity",
                "memos.api.v2.ActivityPayload",
                "memos.api.v2.ActivityMemoCommentPayload",
                "memos.api.v2.ActivityVersionUpdatePayload",
                "memos.api.v2.Inbox",
                "memos.api.v2.MemoRelation",
                // "memos.api.v2.Node",
            ],
            false,
            true,
            None,
        )
        .with_field_attributes(
            &[
                "memos.api.v2.Activity.create_time",
                "memos.api.v2.Inbox.create_time",
                "memos.api.v2.Resource.create_time",
                "memos.api.v2.Memo.create_time",
                "memos.api.v2.Memo.update_time",
                "memos.api.v2.Memo.display_time",
            ],
            &[r#"#[serde(with = "crate::api::time_serde")]"#],
        )
        .with_field_attributes(
            &[
                "memos.api.v2.User.password",
                "memos.api.v2.User.name",
                "memos.api.v2.Memo.nodes",
            ],
            &[r#"#[serde(skip)]"#],
        )
        .with_field_attributes(
            &[
                "memos.api.v2.Memo.creator",
                "memos.api.v2.Memo.display_time",
                "memos.api.v2.Memo.pinned",
                "memos.api.v2.Memo.resources",
                "memos.api.v2.Memo.relations",
            ],
            &["#[serde(default)]"],
        )
        .type_attribute(
            "memos.api.v2.User",
            r"#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .field_attribute(
            "memos.api.v2.Memo.row_status",
            r#"#[serde(with = "crate::api::status_serde")]"#,
        )
        .field_attribute(
            "memos.api.v2.Memo.visibility",
            r#"#[serde(with = "crate::api::visibility_serde")]"#,
        )
        .field_attribute(
            "memos.api.v2.Memo.pinned",
            r#"#[serde(with = "crate::api::bool_serde")]"#,
        )
        .field_attribute(
            "memos.api.v2.User.row_status",
            r#"#[serde(with = "crate::api::status_serde", rename(serialize = "rowStatus"))]"#,
        )
        .field_attribute(
            "memos.api.v2.User.role",
            r#"#[serde(with = "crate::api::role_serde")]"#,
        )
        .field_attribute(
            "memos.api.v2.User.create_time",
            r#"#[serde(with = "crate::api::time_serde", rename(serialize = "createdTs"))]"#,
        )
        .field_attribute(
            "memos.api.v2.User.update_time",
            r#"#[serde(with = "crate::api::time_serde", rename(serialize = "updatedTs"))]"#,
        )
        .field_attribute(
            "memos.api.v2.User.avatar_url",
            r#"#[serde(rename(serialize = "avatarUrl"))]"#,
        )
        .compile(
            &[
                "proto/api/v2/user_service.proto",
                "proto/api/v2/system_service.proto",
                "proto/api/v2/memo_service.proto",
                "proto/api/v2/resource_service.proto",
                "proto/api/v2/tag_service.proto",
                "proto/api/v2/activity_service.proto",
                "proto/api/v2/inbox_service.proto",
                "proto/api/v2/auth_service.proto",
                "proto/api/v2/webhook_service.proto",
                "proto/api/v2/markdown_service.proto",
                "proto/api/v2/memo_relation_service.proto",
            ],
            // https://github.com/googleapis/googleapis.git
            &["proto", "googleapis"],
        )
        .unwrap();

    fs::remove_file("src/api/v2/google.api.rs").unwrap();
    fs::rename("src/api/v2/memos.api.v2.rs", "src/api/v2/mod.rs").unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
}
