#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RowStatus {
    Unspecified = 0,
    Active = 1,
    Archived = 2,
}
impl RowStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            RowStatus::Unspecified => "ROW_STATUS_UNSPECIFIED",
            RowStatus::Active => "ACTIVE",
            RowStatus::Archived => "ARCHIVED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ROW_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "ACTIVE" => Some(Self::Active),
            "ARCHIVED" => Some(Self::Archived),
            _ => None,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct User {
    /// The name of the user.
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    #[serde(rename = "username")]
    pub name: ::prost::alloc::string::String,
    #[prost(int32, tag = "2")]
    pub id: i32,
    #[prost(enumeration = "user::Role", tag = "3")]
    #[serde(with = "crate::pb::role_serde")]
    pub role: i32,
    #[prost(string, tag = "4")]
    pub email: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub nickname: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub password: ::prost::alloc::string::String,
    #[prost(enumeration = "RowStatus", tag = "8")]
    #[serde(with = "crate::pb::status_serde", rename(serialize = "rowStatus"))]
    pub row_status: i32,
    #[prost(message, optional, tag = "9")]
    #[serde(
        with = "crate::pb::time_serde",
        rename(serialize = "createdTs", deserialize = "create_ts")
    )]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "10")]
    #[serde(
        with = "crate::pb::time_serde",
        rename(serialize = "updatedTs", deserialize = "update_ts")
    )]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Nested message and enum types in `User`.
pub mod user {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Role {
        Unspecified = 0,
        Host = 1,
        Admin = 2,
        User = 3,
    }
    impl Role {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Role::Unspecified => "ROLE_UNSPECIFIED",
                Role::Host => "HOST",
                Role::Admin => "ADMIN",
                Role::User => "USER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ROLE_UNSPECIFIED" => Some(Self::Unspecified),
                "HOST" => Some(Self::Host),
                "ADMIN" => Some(Self::Admin),
                "USER" => Some(Self::User),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserRequest {
    /// The name of the user.
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRequest {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserRequest {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
    #[prost(message, optional, tag = "2")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserAccessToken {
    #[prost(string, tag = "1")]
    pub access_token: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub issued_at: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub expires_at: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUserAccessTokensRequest {
    /// The name of the user.
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUserAccessTokensResponse {
    #[prost(message, repeated, tag = "1")]
    pub access_tokens: ::prost::alloc::vec::Vec<UserAccessToken>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserAccessTokenRequest {
    /// The name of the user.
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub expires_at: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserAccessTokenResponse {
    #[prost(message, optional, tag = "1")]
    pub access_token: ::core::option::Option<UserAccessToken>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserAccessTokenRequest {
    /// The name of the user.
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// access_token is the access token to delete.
    #[prost(string, tag = "2")]
    pub access_token: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserAccessTokenResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SystemInfo {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub mode: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub allow_registration: bool,
    #[prost(bool, tag = "4")]
    pub disable_password_login: bool,
    #[prost(string, tag = "5")]
    pub additional_script: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub additional_style: ::prost::alloc::string::String,
    #[prost(int64, tag = "7")]
    pub db_size: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSystemInfoRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSystemInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub system_info: ::core::option::Option<SystemInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSystemInfoRequest {
    /// System info is the updated data.
    #[prost(message, optional, tag = "1")]
    pub system_info: ::core::option::Option<SystemInfo>,
    #[prost(message, optional, tag = "2")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSystemInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub system_info: ::core::option::Option<SystemInfo>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Memo {
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(enumeration = "RowStatus", tag = "2")]
    #[serde(with = "crate::pb::status_serde")]
    pub row_status: i32,
    #[prost(int32, tag = "3")]
    pub creator_id: i32,
    #[prost(int64, tag = "4")]
    pub created_ts: i64,
    #[prost(int64, tag = "5")]
    pub updated_ts: i64,
    #[prost(string, tag = "6")]
    pub content: ::prost::alloc::string::String,
    #[prost(enumeration = "Visibility", tag = "7")]
    pub visibility: i32,
    #[prost(bool, tag = "8")]
    pub pinned: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMemoRequest {
    #[prost(string, tag = "1")]
    pub content: ::prost::alloc::string::String,
    #[prost(enumeration = "Visibility", tag = "2")]
    pub visibility: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMemoResponse {
    #[prost(message, optional, tag = "1")]
    pub memo: ::core::option::Option<Memo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMemosRequest {
    #[prost(int32, tag = "1")]
    pub page: i32,
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// Filter is used to filter memos returned in the list.
    #[prost(string, tag = "3")]
    pub filter: ::prost::alloc::string::String,
    #[prost(int32, optional, tag = "4")]
    pub creator_id: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMemosResponse {
    #[prost(message, repeated, tag = "1")]
    pub memos: ::prost::alloc::vec::Vec<Memo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMemoRequest {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMemoResponse {
    #[prost(message, optional, tag = "1")]
    pub memo: ::core::option::Option<Memo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMemoCommentRequest {
    /// id is the memo id to create comment for.
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(message, optional, tag = "2")]
    pub create: ::core::option::Option<CreateMemoRequest>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMemoCommentResponse {
    #[prost(message, optional, tag = "1")]
    pub memo: ::core::option::Option<Memo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMemoCommentsRequest {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMemoCommentsResponse {
    #[prost(message, repeated, tag = "1")]
    pub memos: ::prost::alloc::vec::Vec<Memo>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Visibility {
    Unspecified = 0,
    Private = 1,
    Protected = 2,
    Public = 3,
}
impl Visibility {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Visibility::Unspecified => "VISIBILITY_UNSPECIFIED",
            Visibility::Private => "PRIVATE",
            Visibility::Protected => "PROTECTED",
            Visibility::Public => "PUBLIC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "VISIBILITY_UNSPECIFIED" => Some(Self::Unspecified),
            "PRIVATE" => Some(Self::Private),
            "PROTECTED" => Some(Self::Protected),
            "PUBLIC" => Some(Self::Public),
            _ => None,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Resource {
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(message, optional, tag = "2")]
    #[serde(with = "crate::pb::time_serde")]
    pub created_ts: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, tag = "3")]
    pub filename: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub external_link: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(int64, tag = "6")]
    pub size: i64,
    #[prost(int32, optional, tag = "7")]
    pub memo_id: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResourceRequest {
    #[prost(string, tag = "1")]
    pub filename: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub external_link: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(int32, optional, tag = "4")]
    pub memo_id: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResourceResponse {
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<Resource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResourcesRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResourcesResponse {
    #[prost(message, repeated, tag = "1")]
    pub resources: ::prost::alloc::vec::Vec<Resource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateResourceRequest {
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<Resource>,
    #[prost(message, optional, tag = "2")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateResourceResponse {
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<Resource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceRequest {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceResponse {}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tag {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The creator of tags.
    /// Format: users/{username}
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpsertTagRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpsertTagResponse {
    #[prost(message, optional, tag = "1")]
    pub tag: ::core::option::Option<Tag>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTagsRequest {
    /// The creator of tags.
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    pub creator: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTagsResponse {
    #[prost(message, repeated, tag = "1")]
    pub tags: ::prost::alloc::vec::Vec<Tag>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTagRequest {
    #[prost(message, optional, tag = "1")]
    pub tag: ::core::option::Option<Tag>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTagResponse {}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Activity {
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(int32, tag = "2")]
    pub creator_id: i32,
    #[prost(string, tag = "3")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub level: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "5")]
    #[serde(with = "crate::pb::time_serde", rename = "create_ts")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "6")]
    pub payload: ::core::option::Option<ActivityPayload>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivityMemoCommentPayload {
    #[prost(int32, tag = "1")]
    pub memo_id: i32,
    #[prost(int32, tag = "2")]
    pub related_memo_id: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivityVersionUpdatePayload {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivityPayload {
    #[prost(message, optional, tag = "1")]
    pub memo_comment: ::core::option::Option<ActivityMemoCommentPayload>,
    #[prost(message, optional, tag = "2")]
    pub version_update: ::core::option::Option<ActivityVersionUpdatePayload>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetActivityRequest {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetActivityResponse {
    #[prost(message, optional, tag = "1")]
    pub activity: ::core::option::Option<Activity>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Inbox {
    /// The name of the inbox.
    /// Format: inboxes/{id}
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Format: users/{username}
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    /// Format: users/{username}
    #[prost(string, tag = "3")]
    pub receiver: ::prost::alloc::string::String,
    #[prost(enumeration = "inbox::Status", tag = "4")]
    pub status: i32,
    #[prost(message, optional, tag = "5")]
    #[serde(with = "crate::pb::time_serde", rename = "create_ts")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(enumeration = "inbox::Type", tag = "6")]
    pub r#type: i32,
    #[prost(int32, optional, tag = "7")]
    pub activity_id: ::core::option::Option<i32>,
}
/// Nested message and enum types in `Inbox`.
pub mod inbox {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Status {
        Unspecified = 0,
        Unread = 1,
        Archived = 2,
    }
    impl Status {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Status::Unspecified => "STATUS_UNSPECIFIED",
                Status::Unread => "UNREAD",
                Status::Archived => "ARCHIVED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATUS_UNSPECIFIED" => Some(Self::Unspecified),
                "UNREAD" => Some(Self::Unread),
                "ARCHIVED" => Some(Self::Archived),
                _ => None,
            }
        }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unspecified = 0,
        MemoComment = 1,
        VersionUpdate = 2,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::MemoComment => "TYPE_MEMO_COMMENT",
                Type::VersionUpdate => "TYPE_VERSION_UPDATE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TYPE_MEMO_COMMENT" => Some(Self::MemoComment),
                "TYPE_VERSION_UPDATE" => Some(Self::VersionUpdate),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListInboxesRequest {
    /// Format: users/{username}
    #[prost(string, tag = "1")]
    pub user: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListInboxesResponse {
    #[prost(message, repeated, tag = "1")]
    pub inboxes: ::prost::alloc::vec::Vec<Inbox>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateInboxRequest {
    #[prost(message, optional, tag = "1")]
    pub inbox: ::core::option::Option<Inbox>,
    #[prost(message, optional, tag = "2")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateInboxResponse {
    #[prost(message, optional, tag = "1")]
    pub inbox: ::core::option::Option<Inbox>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteInboxRequest {
    /// The name of the inbox to delete.
    /// Format: inboxes/{inbox}
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteInboxResponse {}
