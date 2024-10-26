use std::{fs, process::Command};

use proto_builder_trait::tonic::BuilderAttributes;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/*");
    println!(
        "cargo:rustc-env=OUT_DIR={}",
        std::env::var("OUT_DIR").unwrap()
    );

    unsafe {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    }

    // Paths are specified in terms of the Protobuf type name (not the generated Rust type name).
    tonic_build::configure()
        .out_dir("src/api/v1")
        .with_serde(&["memos.api.v1.PageToken"], true, true, None)
        .compile_protos(
            &[
                "proto/api/v1/activity_service.proto",
                "proto/api/v1/auth_service.proto",
                "proto/api/v1/idp_service.proto",
                "proto/api/v1/inbox_service.proto",
                "proto/api/v1/markdown_service.proto",
                "proto/api/v1/memo_relation_service.proto",
                "proto/api/v1/memo_service.proto",
                "proto/api/v1/reaction_service.proto",
                "proto/api/v1/resource_service.proto",
                "proto/api/v1/user_service.proto",
                "proto/api/v1/webhook_service.proto",
                "proto/api/v1/workspace_service.proto",
                "proto/api/v1/workspace_setting_service.proto",
            ],
            // https://github.com/googleapis/googleapis.git
            &["proto", "googleapis"],
        )
        .unwrap();

    tonic_build::configure()
        .out_dir("src/model")
        .with_serde(
            &[
                "memos.store.ResourcePayload",
                "memos.store.ResourcePayload.payload",
                "memos.store.ResourcePayload.S3Object",
                "memos.store.StorageS3Config",
                "memos.store.MemoPayload",
                "memos.store.MemoPayload.Property",
                "memos.store.WorkspaceSetting",
                "memos.store.WorkspaceSetting.value",
                "memos.store.WorkspaceBasicSetting",
                "memos.store.WorkspaceGeneralSetting",
                "memos.store.WorkspaceStorageSetting",
                "memos.store.WorkspaceMemoRelatedSetting",
                "memos.store.WorkspaceCustomProfile",
            ],
            true,
            true,
            None,
        )
        .with_field_attributes(
            &["memos.store.ResourcePayload.S3Object.last_presigned_time"],
            &[r#"#[serde(with = "crate::model::time_serde")]"#],
        )
        .compile_protos(
            &[
                "proto/store/activity.proto",
                "proto/store/idp.proto",
                "proto/store/inbox.proto",
                "proto/store/memo.proto",
                "proto/store/reaction.proto",
                "proto/store/resource.proto",
                "proto/store/user_setting.proto",
                "proto/store/workspace_setting.proto",
            ],
            // https://github.com/googleapis/googleapis.git
            &["proto"],
        )
        .unwrap();

    fs::rename("src/api/v1/memos.api.v1.rs", "src/api/v1/gen.rs").unwrap();
    fs::rename("src/model/memos.store.rs", "src/model/gen.rs").unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
}
