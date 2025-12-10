use crate::actor_store::aws::s3::S3BlobStore;
use crate::actor_store::ActorStore;
use crate::apis::ApiError;
use crate::auth_verifier::AccessStandardIncludeChecks;
use crate::db::DbConn;
use anyhow::{bail, Error, Result};
use aws_sdk_s3::Config;
use rocket::data::Data;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Request, State};
use rsky_common::env::env_bool;
use rsky_common::BadContentTypeError;
use rsky_lexicon::com::atproto::repo::Blob;
use rsky_lexicon::fans::web5::ckb::BlobOutput;
use rsky_repo::types::{BlobConstraint, PreparedBlobRef};
use url::Url;

#[derive(Clone)]
pub struct ContentType {
    pub name: String,
}

/// Used mainly as a way to parse out content-type from request
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentType {
    type Error = BadContentTypeError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.content_type() {
            None => Outcome::Error((
                Status::UnsupportedMediaType,
                BadContentTypeError::MissingType,
            )),
            Some(content_type) => Outcome::Success(ContentType {
                name: content_type.to_string(),
            }),
        }
    }
}

async fn inner_upload_blob(
    auth: AccessStandardIncludeChecks,
    blob: Data<'_>,
    content_type: ContentType,
    s3_config: &State<Config>,
    db: DbConn,
) -> Result<BlobOutput> {
    let requester = auth.access.credentials.unwrap().did.unwrap();

    let actor_store = ActorStore::new(
        requester.clone(),
        S3BlobStore::new(requester.clone(), s3_config.inner().clone()),
        db,
    );

    if !actor_store
        .blob
        .blobstore
        .bucket_exists()
        .await
        .map_err(|e| Error::msg(format!("{:?}", e)))?
    {
        actor_store
            .blob
            .blobstore
            .create_bucket()
            .await
            .map_err(|e| Error::msg(format!("{:?}", e)))?;
    }

    let metadata = actor_store
        .blob
        .upload_blob_and_get_metadata(content_type.name, blob)
        .await?;
    let blobref = actor_store.blob.track_untethered_blob(metadata).await?;

    actor_store
        .blob
        .web5_verify_blob_and_make_permanent(
            requester,
            PreparedBlobRef {
                cid: blobref.get_cid()?,
                mime_type: blobref.get_mime_type().to_string(),
                constraints: BlobConstraint {
                    max_size: None,
                    accept: None,
                },
            },
        )
        .await?;
    let force = env_bool("FORCE_PATH_STYLE").unwrap_or(false);
    if let Ok(mut url) =
        Url::parse(&std::env::var("AWS_ENDPOINT").unwrap_or("http://localhost".to_owned()))
    {
        if force {
            let path = actor_store.blob.blobstore.bucket + "/";
            url.set_path(&path);
        } else if let Some(old_host) = url.host_str() {
            let new_host = actor_store.blob.blobstore.bucket + "." + old_host;
            if url.set_host(Some(&new_host)).is_err() {
                bail!("oss set host failed: {}", new_host);
            }
        } else {
            bail!("env AWS_ENDPOINT host invalid: {}", url.as_str());
        }
        Ok(BlobOutput {
            blob_server: url.as_str().to_string(),
            blob: Blob {
                r#type: Some("blob".to_string()),
                r#ref: Some(blobref.get_cid()?),
                cid: None,
                mime_type: blobref.get_mime_type().to_string(),
                size: blobref.get_size(),
                original: None,
            },
        })
    } else {
        bail!("env AWS_ENDPOINT invalid");
    }
}

#[tracing::instrument(skip_all)]
#[rocket::post("/xrpc/fans.web5.ckb.uploadBlob", data = "<blob>")]
pub async fn upload_blob(
    auth: AccessStandardIncludeChecks,
    blob: Data<'_>,
    content_type: ContentType,
    s3_config: &State<Config>,
    db: DbConn,
) -> Result<Json<BlobOutput>, ApiError> {
    match inner_upload_blob(auth, blob, content_type, s3_config, db).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => {
            tracing::error!("{error:?}");
            Err(ApiError::RuntimeError(Some(error.to_string())))
        }
    }
}
