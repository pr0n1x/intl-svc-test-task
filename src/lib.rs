//! ## Task Description
//!
//! The goal is to develop a backend service for shortening URLs using CQRS
//! (Command Query Responsibility Segregation) and ES (Event Sourcing)
//! approaches. The service should support the following features:
//!
//! ## Functional Requirements
//!
//! ### Creating a short link with a random slug
//!
//! The user sends a long URL, and the service returns a shortened URL with a
//! random slug.
//!
//! ### Creating a short link with a predefined slug
//!
//! The user sends a long URL along with a predefined slug, and the service
//! checks if the slug is unique. If it is unique, the service creates the short
//! link.
//!
//! ### Counting the number of redirects for the link
//!
//! - Every time a user accesses the short link, the click count should
//!   increment.
//! - The click count can be retrieved via an API.
//!
//! ### CQRS+ES Architecture
//!
//! CQRS: Commands (creating links, updating click count) are separated from
//! queries (retrieving link information).
//!
//! Event Sourcing: All state changes (link creation, click count update) must be
//! recorded as events, which can be replayed to reconstruct the system's state.
//!
//! ### Technical Requirements
//!
//! - The service must be built using CQRS and Event Sourcing approaches.
//! - The service must be possible to run in Rust Playground (so no database like
//!   Postgres is allowed)
//! - Public API already written for this task must not be changed (any change to
//!   the public API items must be considered as breaking change).

extern crate url as url_parser;

mod cqrs;
mod gen;
mod base64;
mod string_based_type;
mod owned_borrowed_pair;

#[cfg(test)]
mod test;

use cqrs::store::StoredEventList;
use owned_borrowed_pair::*;

/// All possible errors of the [`UrlShortenerService`].
#[derive(Debug, PartialEq)]
pub enum ShortenerError {
    /// This error occurs when an invalid [`Url`] is provided for shortening.
    InvalidUrl,

    /// This error occurs when an attempt is made to use a slug (custom alias)
    /// that already exists.
    SlugAlreadyInUse,

    /// This error occurs when the provided [`Slug`] does not map to any existing
    /// short link.
    SlugNotFound,
}

/// A unique string (or alias) that represents the shortened version of the
/// URL.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Slug(pub String);

/// The original URL that the short link points to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Url(pub String);

/// Shortened URL representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ShortLink {
    /// A unique string (or alias) that represents the shortened version of the
    /// URL.
    pub slug: Slug,

    /// The original URL that the short link points to.
    pub url: Url,
}

/// Statistics of the [`ShortLink`].
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// [`ShortLink`] to which this [`Stats`] are related.
    pub link: ShortLink,

    /// Count of redirects of the [`ShortLink`].
    pub redirects: u64,
}

/// Commands for CQRS.
pub mod commands {
    use super::{ShortLink, ShortenerError, Slug, Url};

    /// Trait for command handlers.
    pub trait CommandHandler {
        /// Creates a new short link. It accepts the original url and an
        /// optional [`Slug`]. If a [`Slug`] is not provided, the service will generate
        /// one. Returns the newly created [`ShortLink`].
        ///
        /// ## Errors
        ///
        /// See [`ShortenerError`].
        fn handle_create_short_link(
            &mut self,
            url: Url,
            slug: Option<Slug>,
        ) -> Result<ShortLink, ShortenerError>;

        /// Processes a redirection by [`Slug`], returning the associated
        /// [`ShortLink`] or a [`ShortenerError`].
        fn handle_redirect(
            &mut self,
            slug: Slug,
        ) -> Result<ShortLink, ShortenerError>;
    }
}

/// Queries for CQRS
pub mod queries {
    use super::{ShortenerError, Slug, Stats};

    /// Trait for query handlers.
    pub trait QueryHandler {
        /// Returns the [`Stats`] for a specific [`ShortLink`], such as the
        /// number of redirects (clicks).
        ///
        /// [`ShortLink`]: super::ShortLink
        fn get_stats(&self, slug: Slug) -> Result<Stats, ShortenerError>;
    }
}

/// CQRS and Event Sourcing-based service implementation
pub struct UrlShortenerService {
    storage: Box<dyn cqrs::store::EventStore<Stats>>,
    slug_generator: Box<dyn gen::SlugGenerator>,
}

impl UrlShortenerService {
    /// Creates a new instance of the service
    pub fn new(
        storage: Box<dyn cqrs::store::EventStore<Stats>>,
        generator: Box<dyn gen::SlugGenerator>,
    ) -> Self {
        Self { storage, slug_generator: generator }
    }
}

impl commands::CommandHandler for UrlShortenerService {
    fn handle_create_short_link(
        &mut self,
        url: Url,
        slug: Option<Slug>,
    ) -> Result<ShortLink, ShortenerError> {

        if let Err(_) = url_parser::Url::parse(url.as_ref()) {
            return Err(ShortenerError::InvalidUrl)
        }
        
        let slug = match slug {
            Some(slug) => {
                let is_exist = self.storage
                    .is_exist(&slug)
                    .map_err(map_fetch_err_to_shortener_err)?;
                if is_exist {
                    return Err(ShortenerError::SlugAlreadyInUse)
                }
                slug
            }
            None => {
                let mut bump: u16 = 0;
                loop {
                    let generated_slug = self.slug_generator.generate(url.as_ref(), bump);
                    let is_exist = self.storage
                        .is_exist(&generated_slug)
                        .map_err(map_fetch_err_to_shortener_err)?;
                    if !is_exist {
                        break generated_slug
                    }
                    bump += 1;
                    if bump >= u16::MAX {
                        unreachable!("somehow bump reaches it's maximum");
                    }
                }
            }
        };

        // unwrap: i promise, this is a correct event list
        let event_list = StoredEventList::new(&[ShortenerEvent::Create(slug, url)]).unwrap();
        let snapshot = event_list.snapshot();
        // unwrap: there is not type error to handle storage event
        self.storage.commit(event_list).unwrap();

        Ok(snapshot.into_aggregate().link)
    }

    fn handle_redirect(
        &mut self,
        slug: Slug,
    ) -> Result<ShortLink, ShortenerError> {
        let event_list = self
            .storage.fetch(&slug)
            .map_err(map_fetch_err_to_shortener_err)?
            .append_all(&[ShortenerEvent::ShortLinkStatEvent(slug, ShortLinkStatEvent::Redirect)]);
        let snapshot = event_list.snapshot();
        // unwrap: there is not type error to handle storage event
        self.storage.commit(event_list).unwrap();
        Ok(snapshot.into_aggregate().link)
    }
}

impl queries::QueryHandler for UrlShortenerService {
    fn get_stats(&self, slug: Slug) -> Result<Stats, ShortenerError> {
        Ok(self.storage
            .fetch(slug.as_ref())
            .map_err(map_fetch_err_to_shortener_err)?
            .snapshot()
            .into_aggregate())
    }
}

/////////////////////////////////////////////////////////////////

string_based_type!(Slug exists, SlugRef);
string_based_type!(Url exists, UrlRef);

impl cqrs::AggregateIdContract for Slug { type BorrowedAggregateId = SlugRef; }
impl cqrs::AggregateIdRefContract for SlugRef { type OwnedAggregateId = Slug; }

fn map_fetch_err_to_shortener_err(e: cqrs::store::EventStoreError) -> ShortenerError {
    match e {
        cqrs::store::EventStoreError::AggregateIsNotExist => ShortenerError::SlugNotFound,
        _ => Err(e).unwrap(), // any otther error from .snapshot method is unexpected i.e. panic
    }
}

impl cqrs::Aggregate for Stats {
    type Event = ShortenerEvent;
    type Id = Slug;
    type IdRef = SlugRef;
    fn aggregate_type() -> &'static SlugRef {
        "short_link".as_ref()
    }
    fn aggregate_id(&self) -> &SlugRef {
        &self.link.slug
    }
    fn apply(&mut self, event: ShortenerEvent) {
        match event {
            ShortenerEvent::Create(slug, url) => {
                self.link = ShortLink { slug: slug.clone(), url: url.clone() };
                self.redirects = 0;
            }
            ShortenerEvent::ShortLinkStatEvent(slug, stat_event) => {
                if slug.as_str() == self.aggregate_id().as_str() {
                    match stat_event {
                        ShortLinkStatEvent::Redirect => self.redirects += 1,
                    }
                }
            }
        }
    }
}

/// Events aggregated by SLUG
#[derive(Clone, Debug)]
pub enum ShortenerEvent {
    Create(Slug, Url),
    ShortLinkStatEvent(Slug, ShortLinkStatEvent),
}

#[derive(Clone, Debug)]
pub enum ShortLinkStatEvent {
    Redirect
}

impl cqrs::DomainEvent for ShortenerEvent {
    const EVENT_TYPE: &'static str = "ShortenerEvent";
    fn event_name(&self) -> &'static str {
        match self {
            ShortenerEvent::Create(_, _) => "Create",
            ShortenerEvent::ShortLinkStatEvent(_, _) => "ShortLinkStatEvent",
        }
    }
}

impl cqrs::DomainEvent for ShortLinkStatEvent {
    const EVENT_TYPE: &'static str = "ShortLinkStatEvent";
    fn event_name(&self) -> &'static str {
        match self {
            ShortLinkStatEvent::Redirect => "Redirect"
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            link: ShortLink {
                slug: Slug(String::new()),
                url: Url(String::new()),
            },
            redirects: 0,
        }
    }
}

//////////////////////////////////////////
/// ShortenerError impl of Error trait ///
impl core::error::Error for ShortenerError {}
impl core::fmt::Display for ShortenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortenerError::InvalidUrl => write!(f, "invalid url"),
            ShortenerError::SlugAlreadyInUse => write!(f, "slug already in use"),
            ShortenerError::SlugNotFound => write!(f, "slug not found"),
        }
    }
}
