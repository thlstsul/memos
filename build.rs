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
                //TODO other\ rename\ serde
                // "memos.api.v2.User",
                "memos.api.v2.SystemInfo",
                "memos.api.v2.Tag",
            ],
            true,
            true,
            None,
        )
        .with_field_attributes(&[], &[r#"#[serde(with="crate::pb")]"#])
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
