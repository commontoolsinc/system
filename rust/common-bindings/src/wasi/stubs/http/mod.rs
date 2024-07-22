use crate::wasi::bindings::io::streams::{InputStream, OutputStream};
use crate::wasi::TrappableError;
use crate::wasi::{
    bindings::{
        http::types::{self, ErrorCode, StatusCode},
        sync::http::types::{Headers, Method, Scheme, Trailers},
    },
    poll::{Pollable, Subscribe},
};
use crate::wasi::{WasiImpl, WasiView};
use wasmtime::component::Resource;

pub type HttpResult<T> = Result<T, HttpError>;
pub type HttpError = TrappableError<ErrorCode>;

/// The concrete type behind a `wasi:http/types/incoming-request` resource.
pub struct HostIncomingRequest {}

/// The concrete type behind a `wasi:http/types/response-outparam` resource.
pub struct HostResponseOutparam {}

/// The concrete type behind a `wasi:http/types/outgoing-response` resource.
pub struct HostOutgoingResponse {}

/// The concrete type behind a `wasi:http/types/outgoing-request` resource.
pub struct HostOutgoingRequest {}

/// The concrete type behind a `wasi:http/types/request-options` resource.
pub struct HostRequestOptions {}

/// The concrete type behind a `wasi:http/types/incoming-response` resource.
pub struct HostIncomingResponse {}

/// The concrete type behind a `wasi:http/types/fields` resource.
pub enum HostFields {
    /// A reference to the fields of a parent entry.
    Ref,
    /// An owned version of the fields.
    Owned,
}

/// The concrete type behind a `wasi:http/types/future-incoming-response` resource.
pub enum HostFutureIncomingResponse {
    /// A pending response
    Pending,
    /// The response is ready.
    Ready,
    /// The response has been consumed.
    Consumed,
}

/// The concrete type behind a `was:http/types/incoming-body` resource.
pub struct HostIncomingBody {}

/// The concrete type behind the `wasi:io/streams/input-stream` resource returned
/// by `wasi:http/types/incoming-body`'s `stream` method.
pub struct HostIncomingBodyStream {}

#[async_trait::async_trait]
impl Subscribe for HostIncomingBodyStream {
    async fn ready(&mut self) {}
}

/// The concrete type behind a `wasi:http/types/future-trailers` resource.
pub enum HostFutureTrailers {
    /// Trailers aren't here yet.
    Waiting,

    /// Trailers are ready and here they are.
    Done,

    /// Trailers have been consumed by `future-trailers.get`.
    Consumed,
}

#[async_trait::async_trait]
impl Subscribe for HostFutureTrailers {
    async fn ready(&mut self) {}
}

/// The concrete type behind a `wasi:http/types/outgoing-body` resource.
pub struct HostOutgoingBody {}

impl<T> crate::wasi::bindings::http::outgoing_handler::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn handle(
        &mut self,
        _request_id: Resource<HostOutgoingRequest>,
        _options: Option<Resource<types::RequestOptions>>,
    ) -> HttpResult<Resource<HostFutureIncomingResponse>> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn convert_error_code(&mut self, _err: HttpError) -> wasmtime::Result<types::ErrorCode> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn http_error_code(
        &mut self,
        _err: wasmtime::component::Resource<types::IoError>,
    ) -> wasmtime::Result<Option<types::ErrorCode>> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostFields for WasiImpl<T>
where
    T: WasiView,
{
    fn new(&mut self) -> wasmtime::Result<Resource<HostFields>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn from_list(
        &mut self,
        _entries: Vec<(String, Vec<u8>)>,
    ) -> wasmtime::Result<Result<Resource<HostFields>, types::HeaderError>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _fields: Resource<HostFields>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn get(
        &mut self,
        _fields: Resource<HostFields>,
        _name: String,
    ) -> wasmtime::Result<Vec<Vec<u8>>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn has(&mut self, _fields: Resource<HostFields>, _name: String) -> wasmtime::Result<bool> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set(
        &mut self,
        _fields: Resource<HostFields>,
        _name: String,
        _byte_values: Vec<Vec<u8>>,
    ) -> wasmtime::Result<Result<(), types::HeaderError>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn delete(
        &mut self,
        _fields: Resource<HostFields>,
        _name: String,
    ) -> wasmtime::Result<Result<(), types::HeaderError>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn append(
        &mut self,
        _fields: Resource<HostFields>,
        _name: String,
        _value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), types::HeaderError>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn entries(
        &mut self,
        _fields: Resource<HostFields>,
    ) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn clone(&mut self, _fields: Resource<HostFields>) -> wasmtime::Result<Resource<HostFields>> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostIncomingRequest for WasiImpl<T>
where
    T: WasiView,
{
    fn method(&mut self, _id: Resource<HostIncomingRequest>) -> wasmtime::Result<Method> {
        Err(ErrorCode::InternalError(None).into())
    }
    fn path_with_query(
        &mut self,
        _id: Resource<HostIncomingRequest>,
    ) -> wasmtime::Result<Option<String>> {
        Err(ErrorCode::InternalError(None).into())
    }
    fn scheme(&mut self, _id: Resource<HostIncomingRequest>) -> wasmtime::Result<Option<Scheme>> {
        Err(ErrorCode::InternalError(None).into())
    }
    fn authority(
        &mut self,
        _id: Resource<HostIncomingRequest>,
    ) -> wasmtime::Result<Option<String>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn headers(
        &mut self,
        _id: Resource<HostIncomingRequest>,
    ) -> wasmtime::Result<Resource<Headers>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn consume(
        &mut self,
        _id: Resource<HostIncomingRequest>,
    ) -> wasmtime::Result<Result<Resource<HostIncomingBody>, ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _id: Resource<HostIncomingRequest>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostOutgoingRequest for WasiImpl<T>
where
    T: WasiView,
{
    fn new(
        &mut self,
        _headers: Resource<Headers>,
    ) -> wasmtime::Result<Resource<HostOutgoingRequest>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn body(
        &mut self,
        _request: Resource<HostOutgoingRequest>,
    ) -> wasmtime::Result<Result<Resource<HostOutgoingBody>, ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _request: Resource<HostOutgoingRequest>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn method(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
    ) -> wasmtime::Result<Method> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_method(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
        _method: Method,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn path_with_query(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
    ) -> wasmtime::Result<Option<String>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_path_with_query(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
        _path_with_query: Option<String>,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn scheme(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
    ) -> wasmtime::Result<Option<Scheme>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_scheme(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
        _scheme: Option<Scheme>,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn authority(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
    ) -> wasmtime::Result<Option<String>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_authority(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
        _authority: Option<String>,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn headers(
        &mut self,
        _request: wasmtime::component::Resource<types::OutgoingRequest>,
    ) -> wasmtime::Result<wasmtime::component::Resource<Headers>> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostResponseOutparam for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _id: Resource<HostResponseOutparam>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
    fn set(
        &mut self,
        _id: Resource<HostResponseOutparam>,
        _resp: Result<Resource<HostOutgoingResponse>, types::ErrorCode>,
    ) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostIncomingResponse for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _response: Resource<HostIncomingResponse>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn status(
        &mut self,
        _response: Resource<HostIncomingResponse>,
    ) -> wasmtime::Result<StatusCode> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn headers(
        &mut self,
        _response: Resource<HostIncomingResponse>,
    ) -> wasmtime::Result<Resource<Headers>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn consume(
        &mut self,
        _response: Resource<HostIncomingResponse>,
    ) -> wasmtime::Result<Result<Resource<HostIncomingBody>, ()>> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostFutureTrailers for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _id: Resource<HostFutureTrailers>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn subscribe(
        &mut self,
        _index: Resource<HostFutureTrailers>,
    ) -> wasmtime::Result<Resource<Pollable>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn get(
        &mut self,
        _id: Resource<HostFutureTrailers>,
    ) -> wasmtime::Result<Option<Result<Result<Option<Resource<Trailers>>, types::ErrorCode>, ()>>>
    {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostIncomingBody for WasiImpl<T>
where
    T: WasiView,
{
    fn stream(
        &mut self,
        _id: Resource<HostIncomingBody>,
    ) -> wasmtime::Result<Result<Resource<InputStream>, ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn finish(
        &mut self,
        _id: Resource<HostIncomingBody>,
    ) -> wasmtime::Result<Resource<HostFutureTrailers>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _id: Resource<HostIncomingBody>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostOutgoingResponse for WasiImpl<T>
where
    T: WasiView,
{
    fn new(
        &mut self,
        _headers: Resource<Headers>,
    ) -> wasmtime::Result<Resource<HostOutgoingResponse>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn body(
        &mut self,
        _id: Resource<HostOutgoingResponse>,
    ) -> wasmtime::Result<Result<Resource<HostOutgoingBody>, ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn status_code(
        &mut self,
        _id: Resource<HostOutgoingResponse>,
    ) -> wasmtime::Result<types::StatusCode> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_status_code(
        &mut self,
        _id: Resource<HostOutgoingResponse>,
        _status: types::StatusCode,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn headers(
        &mut self,
        _id: Resource<HostOutgoingResponse>,
    ) -> wasmtime::Result<Resource<types::Headers>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _id: Resource<HostOutgoingResponse>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostFutureIncomingResponse for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _id: Resource<HostFutureIncomingResponse>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn get(
        &mut self,
        _id: Resource<HostFutureIncomingResponse>,
    ) -> wasmtime::Result<
        Option<Result<Result<Resource<HostIncomingResponse>, types::ErrorCode>, ()>>,
    > {
        Err(ErrorCode::InternalError(None).into())
    }

    fn subscribe(
        &mut self,
        _id: Resource<HostFutureIncomingResponse>,
    ) -> wasmtime::Result<Resource<Pollable>> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostOutgoingBody for WasiImpl<T>
where
    T: WasiView,
{
    fn write(
        &mut self,
        _id: Resource<HostOutgoingBody>,
    ) -> wasmtime::Result<Result<Resource<OutputStream>, ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn finish(
        &mut self,
        _id: Resource<HostOutgoingBody>,
        _ts: Option<Resource<Trailers>>,
    ) -> HttpResult<()> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _id: Resource<HostOutgoingBody>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
}

impl<T> crate::wasi::bindings::http::types::HostRequestOptions for WasiImpl<T>
where
    T: WasiView,
{
    fn new(&mut self) -> wasmtime::Result<Resource<types::RequestOptions>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn connect_timeout(
        &mut self,
        _opts: Resource<types::RequestOptions>,
    ) -> wasmtime::Result<Option<types::Duration>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_connect_timeout(
        &mut self,
        _opts: Resource<types::RequestOptions>,
        _duration: Option<types::Duration>,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn first_byte_timeout(
        &mut self,
        _opts: Resource<types::RequestOptions>,
    ) -> wasmtime::Result<Option<types::Duration>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_first_byte_timeout(
        &mut self,
        _opts: Resource<types::RequestOptions>,
        _duration: Option<types::Duration>,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn between_bytes_timeout(
        &mut self,
        _opts: Resource<types::RequestOptions>,
    ) -> wasmtime::Result<Option<types::Duration>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn set_between_bytes_timeout(
        &mut self,
        _opts: Resource<types::RequestOptions>,
        _duration: Option<types::Duration>,
    ) -> wasmtime::Result<Result<(), ()>> {
        Err(ErrorCode::InternalError(None).into())
    }

    fn drop(&mut self, _rep: Resource<types::RequestOptions>) -> wasmtime::Result<()> {
        Err(ErrorCode::InternalError(None).into())
    }
}
