use std::path;
use std::{collections::HashMap, io::Cursor};

use image::{io::Reader as ImageReader, ImageFormat, ImageOutputFormat};
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::{
    fs::{self, File, OpenOptions},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tonic::{Request, Response, Status};

use crate::{
    api::{
        resource::{FindResource, WholeResource},
        v2::{
            resource_service_server, CreateResourceRequest, CreateResourceResponse,
            DeleteResourceRequest, DeleteResourceResponse, ListResourcesRequest,
            ListResourcesResponse, Resource, UpdateResourceRequest, UpdateResourceResponse,
        },
    },
    dao::resource::ResourceDao,
    state::AppState,
};

use super::get_current_user;

const RESOURCE_PATH: &str = ".resource_cache";
const THUMBNAIL_IMAGE_PATH: &str = ".thumbnail_cache";

pub struct ResourceService {
    dao: ResourceDao,
}

impl ResourceService {
    pub fn new(state: &AppState) -> Self {
        Self {
            dao: ResourceDao {
                state: state.clone(),
            },
        }
    }

    pub async fn create_resource(&self, create: WholeResource) -> Result<Resource, Error> {
        self.dao
            .create_resource(create)
            .await
            .context(CreateResourceFailed)?
            .context(MaybeCreateResourceFailed)
    }

    pub async fn set_memo_resources(
        &self,
        memo_id: i32,
        new_res_ids: Vec<i32>,
        old_res_ids: Vec<i32>,
    ) -> Result<(), Error> {
        let add_res_ids = new_res_ids
            .iter()
            .filter(|&i| !old_res_ids.contains(i))
            .map(|&i| i)
            .collect();
        let del_res_ids = old_res_ids
            .iter()
            .filter(|&i| !new_res_ids.contains(i))
            .map(|&i| i)
            .collect();

        self.dao
            .set_memo_resources(memo_id, add_res_ids, del_res_ids)
            .await
            .context(SetMemoResourcesFailed)
    }

    pub async fn get_resource(&self, id: i32) -> Result<Resource, Error> {
        let mut rs = self
            .dao
            .list_resources(FindResource {
                id: Some(id),
                ..Default::default()
            })
            .await
            .context(GetResourceFailed)?;
        rs.pop().context(ResourceNotFound { id })
    }

    pub async fn get_whole_resource(&self, id: i32) -> Result<WholeResource, Error> {
        self.dao
            .get_resource(id)
            .await
            .context(GetResourceFailed)?
            .context(ResourceNotFound { id })
    }

    pub async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<Resource>>, Error> {
        self.dao
            .relate_resources(memo_ids)
            .await
            .context(RelateResourcesFailed)
    }

    pub async fn relate_resource(&self, memo_id: i32) -> Result<Vec<Resource>, Error> {
        let rs = self
            .dao
            .relate_resources(vec![memo_id])
            .await
            .context(RelateResourcesFailed)?;
        Ok(rs.into_values().next().unwrap_or(vec![]))
    }

    pub async fn get_resource_stream(
        &self,
        Resource {
            id,
            filename,
            r#type,
            ..
        }: Resource,
        thumbnail: bool,
    ) -> Result<ReaderStream<File>, Error> {
        let filename = format!("{}.{}", id, filename);
        let resource_path = path::Path::new(RESOURCE_PATH).join(&filename);
        let thumbnail_path = path::Path::new(THUMBNAIL_IMAGE_PATH).join(&filename);

        let exists = if thumbnail {
            fs::try_exists(&thumbnail_path).await.unwrap_or(false)
        } else {
            fs::try_exists(&resource_path).await.unwrap_or(false)
        };
        if !exists {
            if thumbnail {
                Self::creator_dir(THUMBNAIL_IMAGE_PATH).await?;
            } else {
                Self::creator_dir(RESOURCE_PATH).await?;
            }

            let WholeResource { blob, .. } = Self::get_whole_resource(self, id).await?;
            if thumbnail {
                let mut bytes = Vec::new();
                {
                    let img = ImageReader::new(Cursor::new(blob))
                        .with_guessed_format()
                        .context(OpenResourceFailed)?
                        .decode()
                        .context(ImageDecodeFailed)?;
                    let img = img.thumbnail(512, 512);
                    let format: ImageOutputFormat = ImageFormat::from_path(&filename)
                        .context(ImageEncodeFailed)?
                        .into();
                    img.write_to(&mut Cursor::new(&mut bytes), format)
                        .context(ImageEncodeFailed)?;
                }
                Self::save_file(&thumbnail_path, &bytes).await?;
            } else {
                Self::save_file(&resource_path, &blob).await?;
            }
        }

        let read_path = if thumbnail {
            thumbnail_path
        } else {
            resource_path
        };

        let read_file = File::open(&read_path).await.context(OpenResourceFailed)?;
        Ok(ReaderStream::new(read_file))
    }

    async fn creator_dir(dir: impl AsRef<path::Path>) -> Result<(), Error> {
        if !fs::try_exists(&dir).await.context(CreateCachedDirFailed)? {
            fs::create_dir(dir).await.context(CreateCachedDirFailed)?;
        }
        Ok(())
    }

    async fn save_file(path: impl AsRef<path::Path>, blob: &[u8]) -> Result<(), Error> {
        let mut resource_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(false)
            .open(&path)
            .await
            .context(OpenResourceFailed)?;

        resource_file
            .write_all(blob)
            .await
            .context(WriteResourceFailed)?;
        Ok(())
    }
}

#[tonic::async_trait]
impl resource_service_server::ResourceService for ResourceService {
    async fn list_resources(
        &self,
        request: Request<ListResourcesRequest>,
    ) -> Result<Response<ListResourcesResponse>, Status> {
        let user = get_current_user(&request)?;
        let resources = self
            .dao
            .list_resources(FindResource {
                creator_id: Some(user.id),
                ..Default::default()
            })
            .await
            .context(ListResourceFailed)?;

        Ok(Response::new(ListResourcesResponse { resources }))
    }

    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> Result<Response<DeleteResourceResponse>, Status> {
        let user = get_current_user(&request)?;
        self.dao
            .delete_resource(request.get_ref().id, user.id)
            .await
            .context(DeleteResourceFailed)?;
        Ok(Response::new(DeleteResourceResponse {}))
    }

    async fn update_resource(
        &self,
        request: Request<UpdateResourceRequest>,
    ) -> Result<Response<UpdateResourceResponse>, Status> {
        todo!()
    }

    async fn create_resource(
        &self,
        request: Request<CreateResourceRequest>,
    ) -> Result<Response<CreateResourceResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create resource: {source}"), context(suffix(false)))]
    CreateResourceFailed { source: crate::dao::Error },
    #[snafu(
        display("Maybe create resource failed, because return none"),
        context(suffix(false))
    )]
    MaybeCreateResourceFailed,
    #[snafu(
        display("Failed to set memo resources: {source}"),
        context(suffix(false))
    )]
    SetMemoResourcesFailed { source: crate::dao::Error },
    #[snafu(display("Failed to get resource: {source}"), context(suffix(false)))]
    GetResourceFailed { source: crate::dao::Error },
    #[snafu(display("Resource not found: {id}"), context(suffix(false)))]
    ResourceNotFound { id: i32 },
    #[snafu(display("Failed to list resource: {source}"), context(suffix(false)))]
    ListResourceFailed { source: crate::dao::Error },
    #[snafu(display("Failed to delete resource: {source}"), context(suffix(false)))]
    DeleteResourceFailed { source: crate::dao::Error },
    #[snafu(
        display("Failed to relate resources: {source}"),
        context(suffix(false))
    )]
    RelateResourcesFailed { source: crate::dao::Error },

    #[snafu(
        display("Failed to create cached dir: {source}"),
        context(suffix(false))
    )]
    CreateCachedDirFailed { source: std::io::Error },
    #[snafu(display("Failed to open resource: {source}"), context(suffix(false)))]
    OpenResourceFailed { source: std::io::Error },
    #[snafu(display("Failed to write resource: {source}"), context(suffix(false)))]
    WriteResourceFailed { source: std::io::Error },
    #[snafu(display("Failed to decode image: {source}"), context(suffix(false)))]
    ImageDecodeFailed { source: image::ImageError },
    #[snafu(
        display("Failed to encode thumbnail: {source}"),
        context(suffix(false))
    )]
    ImageEncodeFailed { source: image::ImageError },
}
