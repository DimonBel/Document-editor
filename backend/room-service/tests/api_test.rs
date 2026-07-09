#[tokio::test]
async fn healthz() {
    let app = axum::Router::new().route("/healthz", axum::routing::get(|| async { "ok" }));
    let r = tower::ServiceExt::oneshot(app, http::Request::builder().uri("/healthz").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), 200);
}
