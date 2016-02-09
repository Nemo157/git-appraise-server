use git2;
use std::fs;
use std::path::{ Path, PathBuf };
use std::borrow::Cow;
use typemap::Key;
use router::Router;
use iron::IronResult;
use iron::request::Request;
use iron::response::Response;
use iron::middleware::{ Handler };
use iron::status;
use hyper::method::Method;
use handler::route::Route;
use error::Error;

pub struct RepositoryContext {
  pub requested_path: PathBuf,
  pub canonical_path: PathBuf,
  pub repository: git2::Repository,
}

impl Key for RepositoryContext {
  type Value = RepositoryContext;
}

pub fn inject_repository_context<H: Handler>(root: &Path, handler: H) -> RepositoryContextHandler<H> {
  RepositoryContextHandler {
    canonical_root: fs::canonicalize(root).unwrap(),
    handler: handler,
  }
}

pub struct RepositoryContextHandler<H: Handler> {
  canonical_root: PathBuf,
  handler: H,
}

impl<H: Handler> Handler for RepositoryContextHandler<H> {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    let requested_path = {
      let router = itry!(req.extensions.get::<Router>().ok_or(Error::MissingExtension), status::InternalServerError);
      PathBuf::from(itry!(router.find("repo").ok_or(Error::MissingPathComponent), status::InternalServerError))
    };
    let full_path = self.canonical_root.join(&requested_path);
    let full_canonical_path = itry!(fs::canonicalize(&full_path), status::NotFound);
    let canonical_path = itry!(full_canonical_path.strip_prefix(&self.canonical_root), status::InternalServerError).to_owned();
    let repository = itry!(git2::Repository::open(self.canonical_root.join(&requested_path)), status::NotFound);
    req.extensions.insert::<RepositoryContext>(RepositoryContext {
      requested_path: requested_path,
      canonical_path: canonical_path,
      repository: repository,
    });
    self.handler.handle(req)
  }
}

impl<'a, H: Handler + Route> Route for RepositoryContextHandler<H> {
  fn method() -> Method { H::method() }
  fn route() -> Cow<'static, str> { ("/*repo".to_owned() + &H::route()).into() }
}