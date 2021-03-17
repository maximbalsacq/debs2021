#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Measurement {
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(float, tag = "2")]
    pub latitude: f32,
    #[prost(float, tag = "3")]
    pub longitude: f32,
    ///Particles < 10µm (particulate matter)
    #[prost(float, tag = "4")]
    pub p1: f32,
    ///Particles < 2.5µm (ultrafine particles)
    #[prost(float, tag = "5")]
    pub p2: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Batch {
    #[prost(int64, tag = "1")]
    pub seq_id: i64,
    ///Set to true when it is the last batch
    #[prost(bool, tag = "2")]
    pub last: bool,
    #[prost(message, repeated, tag = "3")]
    pub current: ::prost::alloc::vec::Vec<Measurement>,
    #[prost(message, repeated, tag = "4")]
    pub lastyear: ::prost::alloc::vec::Vec<Measurement>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Benchmark {
    #[prost(int64, tag = "1")]
    pub id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopKCities {
    #[prost(int32, tag = "1")]
    pub position: i32,
    #[prost(string, tag = "2")]
    pub city: ::prost::alloc::string::String,
    #[prost(int32, tag = "3")]
    pub average_aqi_improvement: i32,
    #[prost(int32, tag = "5")]
    pub current_aqip1: i32,
    #[prost(int32, tag = "6")]
    pub current_aqip2: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultQ1 {
    #[prost(int64, tag = "1")]
    pub benchmark_id: i64,
    #[prost(int64, tag = "2")]
    pub batch_seq_id: i64,
    #[prost(message, repeated, tag = "3")]
    pub topkimproved: ::prost::alloc::vec::Vec<TopKCities>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopKStreaks {
    ///begin of the bucket
    #[prost(int32, tag = "1")]
    pub bucket_from: i32,
    ///end of the bucket
    #[prost(int32, tag = "2")]
    pub bucket_to: i32,
    ///round(float, 3) * 1000 as integer
    #[prost(int32, tag = "3")]
    pub bucket_percent: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultQ2 {
    #[prost(int64, tag = "1")]
    pub benchmark_id: i64,
    #[prost(int64, tag = "2")]
    pub batch_seq_id: i64,
    #[prost(message, repeated, tag = "3")]
    pub histogram: ::prost::alloc::vec::Vec<TopKStreaks>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ping {
    #[prost(int64, tag = "1")]
    pub benchmark_id: i64,
    #[prost(int64, tag = "2")]
    pub correlation_id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BenchmarkConfiguration {
    ///Token from the webapp for authentication
    #[prost(string, tag = "1")]
    pub token: ::prost::alloc::string::String,
    ///Small batches might need different algorithms than large batches
    #[prost(int32, tag = "2")]
    pub batch_size: i32,
    ///chosen by the team, listed in the results
    #[prost(string, tag = "3")]
    pub benchmark_name: ::prost::alloc::string::String,
    ///benchmark type, e.g., test
    #[prost(string, tag = "4")]
    pub benchmark_type: ::prost::alloc::string::String,
    #[prost(enumeration = "benchmark_configuration::Query", repeated, tag = "5")]
    pub queries: ::prost::alloc::vec::Vec<i32>,
}
/// Nested message and enum types in `BenchmarkConfiguration`.
pub mod benchmark_configuration {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Query {
        Q1 = 0,
        Q2 = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Point {
    #[prost(double, tag = "1")]
    pub longitude: f64,
    #[prost(double, tag = "2")]
    pub latitude: f64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Polygon {
    #[prost(message, repeated, tag = "1")]
    pub points: ::prost::alloc::vec::Vec<Point>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Location {
    #[prost(string, tag = "1")]
    pub zipcode: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub city: ::prost::alloc::string::String,
    #[prost(double, tag = "3")]
    pub qkm: f64,
    #[prost(int32, tag = "4")]
    pub population: i32,
    #[prost(message, repeated, tag = "5")]
    pub polygons: ::prost::alloc::vec::Vec<Polygon>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Locations {
    #[prost(message, repeated, tag = "1")]
    pub locations: ::prost::alloc::vec::Vec<Location>,
}
#[doc = r" Generated client implementations."]
pub mod challenger_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct ChallengerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ChallengerClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ChallengerClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        #[doc = "Create a new Benchmark based on the configuration"]
        pub async fn create_new_benchmark(
            &mut self,
            request: impl tonic::IntoRequest<super::BenchmarkConfiguration>,
        ) -> Result<tonic::Response<super::Benchmark>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/Challenger.Challenger/createNewBenchmark");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "Get the polygons of all zip areas in germany based on the benchmarktype"]
        pub async fn get_locations(
            &mut self,
            request: impl tonic::IntoRequest<super::Benchmark>,
        ) -> Result<tonic::Response<super::Locations>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/Challenger.Challenger/getLocations");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Depending on your connectivity you have a latency and throughput."]
        #[doc = " Optionally, we try to account for this by first measuring it."]
        #[doc = " The payload of a Ping corresponds roughly to the payload of a batch and the returning Pong roughly the payload of a Result"]
        #[doc = " This kind of measurement is just for development and experimentation (since it could be easily cheated ;-))"]
        #[doc = " We do not consider that once you deploy your implementation on the VMs in our infrastructure"]
        pub async fn initialize_latency_measuring(
            &mut self,
            request: impl tonic::IntoRequest<super::Benchmark>,
        ) -> Result<tonic::Response<super::Ping>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/Challenger.Challenger/initializeLatencyMeasuring",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn measure(
            &mut self,
            request: impl tonic::IntoRequest<super::Ping>,
        ) -> Result<tonic::Response<super::Ping>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/Challenger.Challenger/measure");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn end_measurement(
            &mut self,
            request: impl tonic::IntoRequest<super::Ping>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/Challenger.Challenger/endMeasurement");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "This marks the starting point of the throughput measurements"]
        pub async fn start_benchmark(
            &mut self,
            request: impl tonic::IntoRequest<super::Benchmark>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/Challenger.Challenger/startBenchmark");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "get the next Batch"]
        pub async fn next_batch(
            &mut self,
            request: impl tonic::IntoRequest<super::Benchmark>,
        ) -> Result<tonic::Response<super::Batch>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/Challenger.Challenger/nextBatch");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "post the result"]
        pub async fn result_q1(
            &mut self,
            request: impl tonic::IntoRequest<super::ResultQ1>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/Challenger.Challenger/resultQ1");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn result_q2(
            &mut self,
            request: impl tonic::IntoRequest<super::ResultQ2>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/Challenger.Challenger/resultQ2");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "This marks the end of the throughput measurements"]
        pub async fn end_benchmark(
            &mut self,
            request: impl tonic::IntoRequest<super::Benchmark>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/Challenger.Challenger/endBenchmark");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for ChallengerClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for ChallengerClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "ChallengerClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod challenger_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ChallengerServer."]
    #[async_trait]
    pub trait Challenger: Send + Sync + 'static {
        #[doc = "Create a new Benchmark based on the configuration"]
        async fn create_new_benchmark(
            &self,
            request: tonic::Request<super::BenchmarkConfiguration>,
        ) -> Result<tonic::Response<super::Benchmark>, tonic::Status>;
        #[doc = "Get the polygons of all zip areas in germany based on the benchmarktype"]
        async fn get_locations(
            &self,
            request: tonic::Request<super::Benchmark>,
        ) -> Result<tonic::Response<super::Locations>, tonic::Status>;
        #[doc = " Depending on your connectivity you have a latency and throughput."]
        #[doc = " Optionally, we try to account for this by first measuring it."]
        #[doc = " The payload of a Ping corresponds roughly to the payload of a batch and the returning Pong roughly the payload of a Result"]
        #[doc = " This kind of measurement is just for development and experimentation (since it could be easily cheated ;-))"]
        #[doc = " We do not consider that once you deploy your implementation on the VMs in our infrastructure"]
        async fn initialize_latency_measuring(
            &self,
            request: tonic::Request<super::Benchmark>,
        ) -> Result<tonic::Response<super::Ping>, tonic::Status>;
        async fn measure(
            &self,
            request: tonic::Request<super::Ping>,
        ) -> Result<tonic::Response<super::Ping>, tonic::Status>;
        async fn end_measurement(
            &self,
            request: tonic::Request<super::Ping>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
        #[doc = "This marks the starting point of the throughput measurements"]
        async fn start_benchmark(
            &self,
            request: tonic::Request<super::Benchmark>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
        #[doc = "get the next Batch"]
        async fn next_batch(
            &self,
            request: tonic::Request<super::Benchmark>,
        ) -> Result<tonic::Response<super::Batch>, tonic::Status>;
        #[doc = "post the result"]
        async fn result_q1(
            &self,
            request: tonic::Request<super::ResultQ1>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
        async fn result_q2(
            &self,
            request: tonic::Request<super::ResultQ2>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
        #[doc = "This marks the end of the throughput measurements"]
        async fn end_benchmark(
            &self,
            request: tonic::Request<super::Benchmark>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ChallengerServer<T: Challenger> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: Challenger> ChallengerServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T, B> Service<http::Request<B>> for ChallengerServer<T>
    where
        T: Challenger,
        B: HttpBody + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/Challenger.Challenger/createNewBenchmark" => {
                    #[allow(non_camel_case_types)]
                    struct createNewBenchmarkSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::BenchmarkConfiguration>
                        for createNewBenchmarkSvc<T>
                    {
                        type Response = super::Benchmark;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BenchmarkConfiguration>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_new_benchmark(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = createNewBenchmarkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/getLocations" => {
                    #[allow(non_camel_case_types)]
                    struct getLocationsSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Benchmark> for getLocationsSvc<T> {
                        type Response = super::Locations;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Benchmark>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_locations(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = getLocationsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/initializeLatencyMeasuring" => {
                    #[allow(non_camel_case_types)]
                    struct initializeLatencyMeasuringSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Benchmark>
                        for initializeLatencyMeasuringSvc<T>
                    {
                        type Response = super::Ping;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Benchmark>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).initialize_latency_measuring(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = initializeLatencyMeasuringSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/measure" => {
                    #[allow(non_camel_case_types)]
                    struct measureSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Ping> for measureSvc<T> {
                        type Response = super::Ping;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Ping>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).measure(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = measureSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/endMeasurement" => {
                    #[allow(non_camel_case_types)]
                    struct endMeasurementSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Ping> for endMeasurementSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Ping>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).end_measurement(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = endMeasurementSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/startBenchmark" => {
                    #[allow(non_camel_case_types)]
                    struct startBenchmarkSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Benchmark> for startBenchmarkSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Benchmark>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).start_benchmark(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = startBenchmarkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/nextBatch" => {
                    #[allow(non_camel_case_types)]
                    struct nextBatchSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Benchmark> for nextBatchSvc<T> {
                        type Response = super::Batch;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Benchmark>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).next_batch(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = nextBatchSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/resultQ1" => {
                    #[allow(non_camel_case_types)]
                    struct resultQ1Svc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::ResultQ1> for resultQ1Svc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResultQ1>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).result_q1(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = resultQ1Svc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/resultQ2" => {
                    #[allow(non_camel_case_types)]
                    struct resultQ2Svc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::ResultQ2> for resultQ2Svc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResultQ2>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).result_q2(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = resultQ2Svc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/Challenger.Challenger/endBenchmark" => {
                    #[allow(non_camel_case_types)]
                    struct endBenchmarkSvc<T: Challenger>(pub Arc<T>);
                    impl<T: Challenger> tonic::server::UnaryService<super::Benchmark> for endBenchmarkSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Benchmark>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).end_benchmark(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = endBenchmarkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Challenger> Clone for ChallengerServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: Challenger> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Challenger> tonic::transport::NamedService for ChallengerServer<T> {
        const NAME: &'static str = "Challenger.Challenger";
    }
}
