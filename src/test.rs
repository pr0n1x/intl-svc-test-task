#![cfg(test)]

use crate::{commands::CommandHandler, cqrs::mem_store, gen, queries::QueryHandler, ShortenerError, UrlRef, UrlShortenerService};


fn create_service() -> UrlShortenerService {
    let storage = Box::new(mem_store::MemEventStore::<super::Stats>::new());
    let shortener = Box::new(gen::SimplestSlugGenerator);
    UrlShortenerService::new(storage, shortener)
}

const INVALID_URL: &'static UrlRef = UrlRef::from_str("http://[:::1]");
const VALID_URL: &'static UrlRef = UrlRef::from_str("https://github.com/rust-lang/rust/issues?labels=E-easy&state=open");

macro_rules! test_url {
    ($x:ident) => { format!("https://github.com/rust-lang/rust/issues?labels=E-easy&state=open&x={}", $x) };
}

#[test]
fn service_handle_create_short_link_on_invalid_url() {
    let mut service = create_service();
    
    let result = service.handle_create_short_link(INVALID_URL.to_owned(), None);
    match result {
        Ok(_) => panic!("invalid url accepted"),
        Err(ShortenerError::InvalidUrl) => {},
        Err(e) => panic!("wrong error type on invalid url: error = {e:?}"),
    }
}

#[test]
fn service_handle_create_short_link_on_valid_url() {
    let mut service = create_service();
    let link = service.handle_create_short_link(VALID_URL.to_owned(), None).unwrap();
    assert_eq!(link.url.borrow(), VALID_URL);
    assert_eq!(link.slug.len(), 8);
}

#[test]
fn service_handle_redirect() {
    let mut service = create_service();
    let link = service.handle_create_short_link(VALID_URL.to_owned(), None).unwrap();
    assert_eq!(link.url.borrow(), VALID_URL);
    assert_eq!(link.slug.len(), 8);

    for _ in [0..10] {
        service.handle_redirect(link.slug.clone()).unwrap();
    }
}

#[test]
fn service_handle_many_links() {
    let mut service = create_service();
    let links = (0..10)
        .map(|x| test_url!(x))
        .map(|url| service.handle_create_short_link(crate::Url(url), None).unwrap())
        .collect::<Vec<_>>();

    for (i, link) in links.iter().enumerate() {
        assert_eq!(test_url!(i), link.url.0);
        assert_eq!(link.slug.len(), 8)
    }

    for link in links.iter() {
        let stats = service.get_stats(link.slug.clone()).unwrap();
        assert_eq!(stats.redirects, 0);
        const REDIRECTS: u64 = 123;
        for _ in 0..REDIRECTS {
            service.handle_redirect(link.slug.clone()).unwrap();
        }
        let stats = service.get_stats(link.slug.clone()).unwrap();
        assert_eq!(stats.redirects, REDIRECTS);
    }
}
