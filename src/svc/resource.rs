use std::path;
use std::sync::Arc;
use std::{collections::HashMap, io::Cursor};

use async_trait::async_trait;
use image::{io::Reader as ImageReader, ImageFormat, ImageOutputFormat};
use snafu::{ensure, OptionExt, ResultExt, Snafu};
use tokio::{
    fs::{self, File, OpenOptions},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tonic::{Request, Response, Status};

use crate::api::v1::gen::GetResourceByUidRequest;
use crate::dao::resource::ResourceRepository;
use crate::dao::workspace::WorkspaceRepository;
use crate::google::api::HttpBody;

use crate::svc::workspace::WorkspaceSettingService;
use crate::{
    api::prefix::ExtractName,
    api::v1::gen::{
        resource_service_server, resource_service_server::ResourceServiceServer,
        CreateResourceRequest, DeleteResourceRequest, GetResourceBinaryRequest, GetResourceRequest,
        ListResourcesRequest, ListResourcesResponse, Resource, SearchResourcesRequest,
        SearchResourcesResponse, UpdateResourceRequest,
    },
    model::resource::{FindResource, Resource as ResourceModel},
};

use super::{RequestExt, Service};

const RESOURCE_PATH: &str = ".resource_cache";
const THUMBNAIL_IMAGE_PATH: &str = ".thumbnail_cache";
const MEBI_BYTE: usize = 1024 * 1024;

#[async_trait]
pub trait ResourceService:
    resource_service_server::ResourceService + Clone + Send + Sync + 'static
{
    fn resource_server(self: Arc<Self>) -> ResourceServiceServer<Self> {
        ResourceServiceServer::from_arc(self)
    }

    async fn set_resources_memo(
        &self,
        memo_id: i32,
        new_res_ids: Vec<i32>,
        old_res_ids: Vec<i32>,
    ) -> Result<(), Error>;
    async fn get_resource_by_id(&self, id: i32) -> Result<ResourceModel, Error>;
    async fn get_whole_resource(&self, id: i32) -> Result<ResourceModel, Error>;
    async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<ResourceModel>>, Error>;
    async fn relate_resource(&self, memo_id: i32) -> Result<Vec<ResourceModel>, Error>;
    async fn get_resource_stream(
        &self,
        id: i32,
        filename: String,
        thumbnail: bool,
    ) -> Result<ReaderStream<File>, Error>;
}

#[async_trait]
impl<R: ResourceRepository + WorkspaceRepository> ResourceService for Service<R> {
    async fn set_resources_memo(
        &self,
        memo_id: i32,
        new_res_ids: Vec<i32>,
        old_res_ids: Vec<i32>,
    ) -> Result<(), Error> {
        let add_res_ids = new_res_ids
            .iter()
            .filter(|&i| !old_res_ids.contains(i))
            .copied()
            .collect();
        let del_res_ids = old_res_ids
            .iter()
            .filter(|&i| !new_res_ids.contains(i))
            .copied()
            .collect();

        Ok(self
            .repo
            .set_resources_memo(memo_id, add_res_ids, del_res_ids)
            .await?)
    }

    async fn get_resource_by_id(&self, id: i32) -> Result<ResourceModel, Error> {
        let mut rs = self
            .repo
            .list_resources(FindResource {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        rs.pop().context(ResourceNotFound)
    }

    async fn get_whole_resource(&self, id: i32) -> Result<ResourceModel, Error> {
        self.repo.get_resource(id).await?.context(ResourceNotFound)
    }

    async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<ResourceModel>>, Error> {
        Ok(self.repo.relate_resources(memo_ids).await?)
    }

    async fn relate_resource(&self, memo_id: i32) -> Result<Vec<ResourceModel>, Error> {
        let rs = self.repo.relate_resources(vec![memo_id]).await?;
        Ok(rs.into_values().next().unwrap_or(vec![]))
    }

    async fn get_resource_stream(
        &self,
        id: i32,
        filename: String,
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
                creator_dir(THUMBNAIL_IMAGE_PATH).await?;
            } else {
                creator_dir(RESOURCE_PATH).await?;
            }

            let ResourceModel { blob, .. } = Self::get_whole_resource(self, id).await?;
            if thumbnail {
                let mut bytes = Vec::new();
                {
                    let img = ImageReader::new(Cursor::new(blob))
                        .with_guessed_format()
                        .context(OpenResource)?
                        .decode()
                        .context(ImageDecode)?;
                    let img = img.thumbnail(512, 512);
                    let format: ImageOutputFormat = ImageFormat::from_path(&filename)
                        .context(ImageEncode)?
                        .into();
                    img.write_to(&mut Cursor::new(&mut bytes), format)
                        .context(ImageEncode)?;
                }
                save_file(&thumbnail_path, &bytes).await?;
            } else {
                save_file(&resource_path, &blob).await?;
            }
        }

        let read_path = if thumbnail {
            thumbnail_path
        } else {
            resource_path
        };

        let read_file = File::open(&read_path).await.context(OpenResource)?;
        Ok(ReaderStream::new(read_file))
    }
}

async fn creator_dir(dir: impl AsRef<path::Path>) -> Result<(), Error> {
    if !fs::try_exists(&dir).await.context(CreateCachedDir)? {
        fs::create_dir(dir).await.context(CreateCachedDir)?;
    }
    Ok(())
}

async fn save_file(path: impl AsRef<path::Path>, blob: &[u8]) -> Result<(), Error> {
    let mut resource_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(false)
        .truncate(true)
        .open(&path)
        .await
        .context(OpenResource)?;

    resource_file.write_all(blob).await.context(WriteResource)?;
    Ok(())
}

#[tonic::async_trait]
impl<R: ResourceRepository + WorkspaceRepository> resource_service_server::ResourceService
    for Service<R>
{
    async fn list_resources(
        &self,
        request: Request<ListResourcesRequest>,
    ) -> Result<Response<ListResourcesResponse>, Status> {
        let user = request.get_current_user()?;
        let resources = self
            .repo
            .list_resources(FindResource {
                creator_id: Some(user.id),
                ..Default::default()
            })
            .await?;
        let resources = resources.into_iter().map(|r| r.into()).collect();

        Ok(Response::new(ListResourcesResponse { resources }))
    }

    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> Result<Response<()>, Status> {
        let user = request.get_current_user()?;
        let id = request.get_ref().get_id()?;
        self.repo.delete_resource(id, user.id).await?;
        Ok(Response::new(()))
    }

    async fn get_resource(
        &self,
        request: Request<GetResourceRequest>,
    ) -> Result<Response<Resource>, Status> {
        let id = request.get_ref().get_id()?;
        let res = self.get_resource_by_id(id).await?;
        Ok(Response::new(res.into()))
    }

    async fn get_resource_by_uid(
        &self,
        request: Request<GetResourceByUidRequest>,
    ) -> Result<Response<Resource>, Status> {
        todo!()
    }

    async fn update_resource(
        &self,
        request: Request<UpdateResourceRequest>,
    ) -> Result<Response<Resource>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn create_resource(
        &self,
        request: Request<CreateResourceRequest>,
    ) -> Result<Response<Resource>, Status> {
        let user = request.get_current_user()?;

        if let Some(resource) = &request.get_ref().resource {
            let mut create: ResourceModel = resource.clone().into();
            create.creator_id = user.id;

            let size = create.blob.len();
            let limit = self.get_upload_size_limit().await;
            let max_upload_size_bytes = limit * MEBI_BYTE;
            ensure!(max_upload_size_bytes > size, FileSizeLimit { size: limit });

            // 默认保存到 turso
            let resource = self
                .repo
                .create_resource(create)
                .await?
                .context(MaybeCreateResource)?;
            Ok(Response::new(resource.into()))
        } else {
            Err(Status::data_loss("null request"))
        }
    }

    async fn search_resources(
        &self,
        request: Request<SearchResourcesRequest>,
    ) -> Result<Response<SearchResourcesResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn get_resource_binary(
        &self,
        request: Request<GetResourceBinaryRequest>,
    ) -> Result<Response<HttpBody>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(
        display("Maybe create resource failed, because return none"),
        context(suffix(false))
    )]
    MaybeCreateResource,
    #[snafu(context(false))]
    SetMemoResources {
        source: crate::dao::resource::SetResourceError,
    },
    #[snafu(context(false))]
    GetResource {
        source: crate::dao::resource::GetResourceError,
    },
    #[snafu(display("Resource not found"), context(suffix(false)))]
    ResourceNotFound,
    #[snafu(context(false))]
    ListResource {
        source: crate::dao::resource::ListResourceError,
    },
    #[snafu(context(false))]
    DeleteResource {
        source: crate::dao::resource::DeleteResourceError,
    },
    #[snafu(context(false))]
    RelateResources {
        source: crate::dao::resource::RelateResourceError,
    },

    #[snafu(
        display("File size exceeds allowed limit of {size} MiB"),
        context(suffix(false))
    )]
    FileSizeLimit { size: usize },
    #[snafu(
        display("Failed to create cached dir: {source}"),
        context(suffix(false))
    )]
    CreateCachedDir { source: std::io::Error },
    #[snafu(display("Failed to open resource: {source}"), context(suffix(false)))]
    OpenResource { source: std::io::Error },
    #[snafu(display("Failed to write resource: {source}"), context(suffix(false)))]
    WriteResource { source: std::io::Error },
    #[snafu(display("Failed to decode image: {source}"), context(suffix(false)))]
    ImageDecode { source: image::ImageError },
    #[snafu(
        display("Failed to encode thumbnail: {source}"),
        context(suffix(false))
    )]
    ImageEncode { source: image::ImageError },
}
