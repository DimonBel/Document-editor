"""Test the gateway error contract and health endpoint."""
from fastapi.testclient import TestClient
from gateway.app.main import app


def test_healthz_ok():
    c = TestClient(app)
    r = c.get("/healthz")
    assert r.status_code == 200
    body = r.json()
    assert body["status"] == "ok"
    assert "service" in body


def test_jwks_endpoint_returns_keys():
    c = TestClient(app)
    r = c.get("/.well-known/jwks.json")
    assert r.status_code == 200
    body = r.json()
    assert "keys" in body
    # Implementation returns an empty list until a key is requested; check the contract.
    assert isinstance(body["keys"], list)


def test_unknown_service_proxy_returns_404_problem():
    c = TestClient(app)
    r = c.get("/api/v1/does-not-exist/healthz")
    assert r.status_code == 404
    assert r.headers.get("content-type", "").startswith("application/problem+json")
    body = r.json()
    assert body["status"] == 404
    assert "unknown" in body["title"].lower() or "does-not-exist" in (body.get("detail") or "")


def test_login_returns_access_token():
    c = TestClient(app)
    r = c.post("/auth/login", json={"username": "alice", "password": "x"})
    assert r.status_code == 200
    body = r.json()
    assert body["token_type"] == "Bearer"
    assert body["expires_in"] == 900
    assert body["access_token"]


def test_internal_token_endpoint():
    c = TestClient(app)
    r = c.post("/auth/internal", json={"service": "doc-service"})
    assert r.status_code == 200
    body = r.json()
    assert body["access_token"]
    assert body["expires_in"] == 60
