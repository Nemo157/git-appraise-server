use std::fmt;
use std::time::Duration;
use std::borrow::Cow;
use std::path::Path;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use mime::Mime;
use iron::headers::EntityTag;
use iron::request::Request;

#[macro_export]
macro_rules! file {
  ($x:expr) => ({
    let bytes = include_bytes!($x);
    $crate::handler::utils::File(
      $crate::handler::utils::mime($x),
      ::iron::headers::EntityTag::strong(sha1!(bytes as &[u8])),
      ::std::borrow::Cow::Borrowed(bytes))
  });
}

#[macro_export]
macro_rules! sha1 {
  ($($x:expr),*) => ({
    use ::crypto::digest::Digest;
    let mut hasher = ::crypto::sha1::Sha1::new();
    $(hasher.input(::std::convert::AsRef::<[u8]>::as_ref($x));)*
    hasher.result_str()
  });
}

#[macro_export]
macro_rules! versioned_sha1 {
  () => ({
    sha1!(env!("CARGO_PKG_VERSION"))
  });
  ($($x:expr),+) => ({
    sha1!(env!("CARGO_PKG_VERSION"), $($x),*)
  });
}

#[derive(Clone)]
pub struct File(pub Mime, pub EntityTag, pub Cow<'static, [u8]>);

impl fmt::Debug for File {
  fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
    write!(w, "File{:?}", (&self.0, &self.1, &self.2.len()))
  }
}


pub fn mime(path: &str) -> Mime {
  match Path::new(path).extension().and_then(|s| s.to_str()) {
    Some("css") => mime!(Text/Css),
    Some("js") => mime!(Text/Javascript),
    None | Some(_) => mime!(Application/("octet-stream")),
  }
}

pub fn sha1<T: AsRef<[u8]>>(file: T) -> String {
  let mut hasher = Sha1::new();
  hasher.input(file.as_ref());
  hasher.result_str()
}

pub trait CacheMatches {
  fn cache_matches(&self, etag: &EntityTag) -> bool;
}

// In debug mode assume the etag never matches so we
// don't have to bump version numbers for dynamic content
// (hacky detection, see https://users.rust-lang.org/t/1098)
#[cfg(debug_assertions)]
impl<'a, 'b> CacheMatches for Request<'a, 'b> {
  fn cache_matches(&self, _etag: &EntityTag) -> bool {
    false
  }
}

#[cfg(not(debug_assertions))]
impl<'a, 'b> CacheMatches for Request<'a, 'b> {
  fn cache_matches(&self, etag: &EntityTag) -> bool {
    use iron::headers::IfNoneMatch;
    if let Some(&IfNoneMatch::Items(ref items)) = self.headers.get() {
      if items.len() == 1 && items[0] == *etag {
        return true;
      }
    }
    false
  }
}

#[cfg(debug_assertions)] use iron::modifiers::Header;
#[cfg(debug_assertions)] use iron::headers::Vary;
#[cfg(debug_assertions)] use unicase::UniCase;
// Should return () once https://github.com/reem/rust-modifier/pull/19 is merged
#[cfg(debug_assertions)]
pub fn cache_headers_for(_entity_tag: &EntityTag, _duration: Duration) -> Header<Vary> {
  Header(Vary::Items(vec![
    UniCase("accept-encoding".to_owned()),
  ]))
}

#[cfg(not(debug_assertions))] use iron::modifiers::Header;
#[cfg(not(debug_assertions))] use iron::headers::{ ETag, CacheControl, CacheDirective, Vary };
#[cfg(not(debug_assertions))] use unicase::UniCase;
// Where's my abstract return types....
#[cfg(not(debug_assertions))]
pub fn cache_headers_for(entity_tag: &EntityTag, duration: Duration)
  -> (Header<CacheControl>, Header<ETag>, Header<Vary>)
{
  (
    Header(CacheControl(vec![
      CacheDirective::Public,
      CacheDirective::MaxAge(duration.as_secs() as u32),
    ])),
    Header(ETag(entity_tag.clone())),
    Header(Vary::Items(vec![
      UniCase("accept-encoding".to_owned()),
    ])),
  )
}
