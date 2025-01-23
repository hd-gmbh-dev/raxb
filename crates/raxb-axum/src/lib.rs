#![doc = include_str!("../README.md")]
/*
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
*/

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use hyper::{HeaderMap, StatusCode};
use raxb::de::XmlDeserialize;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
#[must_use]
pub struct RaxbXml<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for RaxbXml<T>
where
    T: XmlDeserialize,
    S: Send + Sync,
{
    type Rejection = RaxbXmlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if xml_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            RaxbXmlSource::from_bytes(bytes).map(|RaxbXmlSource(xml, _)| Self(xml))
        } else {
            Err(MissingXmlContentType.into())
        }
    }
}

#[derive(Debug, Clone, Default)]
#[must_use]
pub struct RaxbXmlSource<T>(pub T, pub Bytes);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for RaxbXmlSource<T>
where
    T: XmlDeserialize,
    S: Send + Sync,
{
    type Rejection = RaxbXmlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if xml_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            Self::from_bytes(bytes)
        } else {
            Err(MissingXmlContentType.into())
        }
    }
}

impl<T> RaxbXmlSource<T>
where
    T: XmlDeserialize,
{
    /// Construct a `RaxbXmlSource<T>` from a byte slice. Most users should prefer to use the `FromRequest` impl
    /// but special cases may require first extracting a `Request` into `Bytes` then optionally
    /// constructing a `RaxbXmlSource<T>`.
    pub fn from_bytes(bytes: Bytes) -> Result<Self, RaxbXmlRejection> {
        raxb::de::from_reader(&*bytes)
            .map(|xml| Self(xml, bytes))
            .map_err(From::from)
    }
}

#[derive(Debug, Error)]
pub enum RaxbXmlRejection {
    #[error(transparent)]
    MissingXmlContentType(#[from] MissingXmlContentType),
    #[error(transparent)]
    AxumError(#[from] axum::extract::rejection::BytesRejection),
    #[error(transparent)]
    RaxbError(#[from] raxb::de::XmlDeserializeError),
}

impl IntoResponse for RaxbXmlRejection {
    fn into_response(self) -> Response {
        StatusCode::BAD_REQUEST.into_response()
    }
}

fn xml_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(hyper::header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_xml_content_type = mime.type_() == "application"
        && (mime.subtype() == "xml" || mime.suffix().is_some_and(|name| name == "xml"));

    is_xml_content_type
}

#[derive(Debug, Clone, Copy, Default, Error)]
#[error("missing or wrong content-type, must be application/xml")]
pub struct MissingXmlContentType;

impl IntoResponse for MissingXmlContentType {
    fn into_response(self) -> Response {
        StatusCode::BAD_REQUEST.into_response()
    }
}
