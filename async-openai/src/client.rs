use std::{pin::Pin, time::Duration};

use bytes::Bytes;
use futures::{stream::StreamExt, Stream};
use reqwest_middleware_eventsource::{Event, EventSource, RequestBuilderExt};
use reqwest_retry::RetryTransientMiddleware;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    config::{Config, OpenAIConfig},
    error::{map_deserialization_error, OpenAIError},
    file::Files,
    image::Images,
    moderation::Moderations,
    Assistants, Audio, AuditLogs, Batches, Chat, Completions, Embeddings, FineTuning, Invites,
    Models, Projects, Threads, Users, VectorStores,
};

#[derive(Debug, Clone, Default)]
/// Client is a container for config, backoff and http_client
/// used to make API calls.
pub struct Client<C: Config> {
    http_client: reqwest_middleware::ClientWithMiddleware,
    config: C,
}

impl Client<OpenAIConfig> {
    /// Client with default [OpenAIConfig]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C: Config> Client<C> {
    /// Create client with a custom HTTP client, OpenAI config, and backoff.
    pub fn build(http_client: reqwest_middleware::ClientWithMiddleware, config: C) -> Self {
        Self {
            http_client,
            config,
        }
    }

    /// Create client with [OpenAIConfig] or [crate::config::AzureConfig]
    pub fn with_config(config: C) -> Self {
        Self {
            http_client: reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
                .with(RetryTransientMiddleware::new_with_policy(
                    reqwest_retry::policies::ExponentialBackoff::builder()
                        .base(2)
                        .jitter(reqwest_retry::Jitter::Full)
                        .retry_bounds(Duration::from_millis(500), Duration::from_secs(60))
                        .build_with_total_retry_duration(Duration::from_secs(900)),
                ))
                .build(),
            config,
        }
    }

    /// Provide your own [client] to make HTTP requests with.
    ///
    /// [client]: reqwest_middleware::ClientWithMiddleware
    pub fn with_http_client(
        mut self,
        http_client: reqwest_middleware::ClientWithMiddleware,
    ) -> Self {
        self.http_client = http_client;
        self
    }

    // API groups

    /// To call [Models] group related APIs using this client.
    pub fn models(&self) -> Models<C> {
        Models::new(self)
    }

    /// To call [Completions] group related APIs using this client.
    pub fn completions(&self) -> Completions<C> {
        Completions::new(self)
    }

    /// To call [Chat] group related APIs using this client.
    pub fn chat(&self) -> Chat<C> {
        Chat::new(self)
    }

    /// To call [Images] group related APIs using this client.
    pub fn images(&self) -> Images<C> {
        Images::new(self)
    }

    /// To call [Moderations] group related APIs using this client.
    pub fn moderations(&self) -> Moderations<C> {
        Moderations::new(self)
    }

    /// To call [Files] group related APIs using this client.
    pub fn files(&self) -> Files<C> {
        Files::new(self)
    }

    /// To call [FineTuning] group related APIs using this client.
    pub fn fine_tuning(&self) -> FineTuning<C> {
        FineTuning::new(self)
    }

    /// To call [Embeddings] group related APIs using this client.
    pub fn embeddings(&self) -> Embeddings<C> {
        Embeddings::new(self)
    }

    /// To call [Audio] group related APIs using this client.
    pub fn audio(&self) -> Audio<C> {
        Audio::new(self)
    }

    /// To call [Assistants] group related APIs using this client.
    pub fn assistants(&self) -> Assistants<C> {
        Assistants::new(self)
    }

    /// To call [Threads] group related APIs using this client.
    pub fn threads(&self) -> Threads<C> {
        Threads::new(self)
    }

    /// To call [VectorStores] group related APIs using this client.
    pub fn vector_stores(&self) -> VectorStores<C> {
        VectorStores::new(self)
    }

    /// To call [Batches] group related APIs using this client.
    pub fn batches(&self) -> Batches<C> {
        Batches::new(self)
    }

    /// To call [AuditLogs] group related APIs using this client.
    pub fn audit_logs(&self) -> AuditLogs<C> {
        AuditLogs::new(self)
    }

    /// To call [Invites] group related APIs using this client.
    pub fn invites(&self) -> Invites<C> {
        Invites::new(self)
    }

    /// To call [Users] group related APIs using this client.
    pub fn users(&self) -> Users<C> {
        Users::new(self)
    }

    /// To call [Projects] group related APIs using this client.
    pub fn projects(&self) -> Projects<C> {
        Projects::new(self)
    }

    pub fn config(&self) -> &C {
        &self.config
    }

    /// Make a GET request to {path} and deserialize the response body
    pub(crate) async fn get<O>(&self, path: &str) -> Result<O, OpenAIError>
    where
        O: DeserializeOwned,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .get(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .build()?)
        };

        self.execute(request_maker).await
    }

    /// Make a GET request to {path} with given Query and deserialize the response body
    pub(crate) async fn get_with_query<Q, O>(&self, path: &str, query: &Q) -> Result<O, OpenAIError>
    where
        O: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .get(self.config.url(path))
                .query(&self.config.query())
                .query(query)
                .headers(self.config.headers())
                .build()?)
        };

        self.execute(request_maker).await
    }

    /// Make a DELETE request to {path} and deserialize the response body
    pub(crate) async fn delete<O>(&self, path: &str) -> Result<O, OpenAIError>
    where
        O: DeserializeOwned,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .delete(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .build()?)
        };

        self.execute(request_maker).await
    }

    /// Make a GET request to {path} and return the response body
    pub(crate) async fn get_raw(&self, path: &str) -> Result<Bytes, OpenAIError> {
        let request_maker = || async {
            Ok(self
                .http_client
                .get(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .build()?)
        };

        self.execute_raw(request_maker).await
    }

    /// Make a POST request to {path} and return the response body
    pub(crate) async fn post_raw<I>(&self, path: &str, request: I) -> Result<Bytes, OpenAIError>
    where
        I: Serialize,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .post(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .json(&request)
                .build()?)
        };

        self.execute_raw(request_maker).await
    }

    /// Make a POST request to {path} and deserialize the response body
    pub(crate) async fn post<I, O>(&self, path: &str, request: I) -> Result<O, OpenAIError>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .post(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .json(&request)
                .build()?)
        };

        self.execute(request_maker).await
    }

    /// POST a form at {path} and return the response body
    pub(crate) async fn post_form_raw<F>(&self, path: &str, form: F) -> Result<Bytes, OpenAIError>
    where
        reqwest::multipart::Form: async_convert::TryFrom<F, Error = OpenAIError>,
        F: Clone,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .post(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .multipart(async_convert::TryFrom::try_from(form.clone()).await?)
                .build()?)
        };

        self.execute_raw(request_maker).await
    }

    /// POST a form at {path} and deserialize the response body
    pub(crate) async fn post_form<O, F>(&self, path: &str, form: F) -> Result<O, OpenAIError>
    where
        O: DeserializeOwned,
        reqwest::multipart::Form: async_convert::TryFrom<F, Error = OpenAIError>,
        F: Clone,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .post(self.config.url(path))
                .query(&self.config.query())
                .headers(self.config.headers())
                .multipart(async_convert::TryFrom::try_from(form.clone()).await?)
                .build()?)
        };

        self.execute(request_maker).await
    }

    /// Execute a HTTP request and retry on rate limit
    ///
    /// request_maker serves one purpose: to be able to create request again
    /// to retry API call after getting rate limited. request_maker is async because
    /// reqwest::multipart::Form is created by async calls to read files for uploads.
    async fn execute_raw<M, Fut>(&self, request_maker: M) -> Result<Bytes, OpenAIError>
    where
        M: Fn() -> Fut,
        Fut: core::future::Future<Output = Result<reqwest::Request, OpenAIError>>,
    {
        let client = self.http_client.clone();

        let request = request_maker().await?;
        let response = client.execute(request).await.map_err(OpenAIError::from)?;
        let bytes = response.bytes().await.map_err(OpenAIError::Reqwest)?;

        Ok(bytes)
    }

    /// Execute a HTTP request and retry on rate limit
    ///
    /// request_maker serves one purpose: to be able to create request again
    /// to retry API call after getting rate limited. request_maker is async because
    /// reqwest::multipart::Form is created by async calls to read files for uploads.
    async fn execute<O, M, Fut>(&self, request_maker: M) -> Result<O, OpenAIError>
    where
        O: DeserializeOwned,
        M: Fn() -> Fut,
        Fut: core::future::Future<Output = Result<reqwest::Request, OpenAIError>>,
    {
        let bytes = self.execute_raw(request_maker).await?;

        let response: O = serde_json::from_slice(bytes.as_ref())
            .map_err(|e| map_deserialization_error(e, bytes.as_ref()))?;

        Ok(response)
    }

    /// Make HTTP POST request to receive SSE
    pub(crate) async fn post_stream<I, O>(
        &self,
        path: &str,
        request: I,
    ) -> Pin<Box<dyn Stream<Item = Result<O, OpenAIError>> + Send>>
    where
        I: Serialize,
        O: DeserializeOwned + std::marker::Send + 'static,
    {
        let event_source = self
            .http_client
            .post(self.config.url(path))
            .query(&self.config.query())
            .headers(self.config.headers())
            .json(&request)
            .eventsource()
            .unwrap();

        stream(event_source).await
    }

    pub(crate) async fn post_stream_mapped_raw_events<I, O>(
        &self,
        path: &str,
        request: I,
        event_mapper: impl Fn(eventsource_stream::Event) -> Result<O, OpenAIError> + Send + 'static,
    ) -> Pin<Box<dyn Stream<Item = Result<O, OpenAIError>> + Send>>
    where
        I: Serialize,
        O: DeserializeOwned + std::marker::Send + 'static,
    {
        let event_source = self
            .http_client
            .post(self.config.url(path))
            .query(&self.config.query())
            .headers(self.config.headers())
            .json(&request)
            .eventsource()
            .unwrap();

        stream_mapped_raw_events(event_source, event_mapper).await
    }

    /// Make HTTP GET request to receive SSE
    pub(crate) async fn _get_stream<Q, O>(
        &self,
        path: &str,
        query: &Q,
    ) -> Pin<Box<dyn Stream<Item = Result<O, OpenAIError>> + Send>>
    where
        Q: Serialize + ?Sized,
        O: DeserializeOwned + std::marker::Send + 'static,
    {
        let event_source = self
            .http_client
            .get(self.config.url(path))
            .query(query)
            .query(&self.config.query())
            .headers(self.config.headers())
            .eventsource()
            .unwrap();

        stream(event_source).await
    }
}

/// Request which responds with SSE.
/// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format)
pub(crate) async fn stream<O>(
    mut event_source: EventSource,
) -> Pin<Box<dyn Stream<Item = Result<O, OpenAIError>> + Send>>
where
    O: DeserializeOwned + std::marker::Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(ev) = event_source.next().await {
            match ev {
                Err(e) => {
                    if let Err(_e) = tx.send(Err(OpenAIError::StreamError(e.to_string()))) {
                        // rx dropped
                        break;
                    }
                }
                Ok(event) => match event {
                    Event::Message(message) => {
                        if message.data == "[DONE]" {
                            break;
                        }

                        let response = match serde_json::from_str::<O>(&message.data) {
                            Err(e) => Err(map_deserialization_error(e, message.data.as_bytes())),
                            Ok(output) => Ok(output),
                        };

                        if let Err(_e) = tx.send(response) {
                            // rx dropped
                            break;
                        }
                    }
                    Event::Open => continue,
                },
            }
        }

        event_source.close();
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}

pub(crate) async fn stream_mapped_raw_events<O>(
    mut event_source: EventSource,
    event_mapper: impl Fn(eventsource_stream::Event) -> Result<O, OpenAIError> + Send + 'static,
) -> Pin<Box<dyn Stream<Item = Result<O, OpenAIError>> + Send>>
where
    O: DeserializeOwned + std::marker::Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(ev) = event_source.next().await {
            match ev {
                Err(e) => {
                    if let Err(_e) = tx.send(Err(OpenAIError::StreamError(e.to_string()))) {
                        // rx dropped
                        break;
                    }
                }
                Ok(event) => match event {
                    Event::Message(message) => {
                        let mut done = false;

                        if message.data == "[DONE]" {
                            done = true;
                        }

                        let response = event_mapper(message);

                        if let Err(_e) = tx.send(response) {
                            // rx dropped
                            break;
                        }

                        if done {
                            break;
                        }
                    }
                    Event::Open => continue,
                },
            }
        }

        event_source.close();
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
