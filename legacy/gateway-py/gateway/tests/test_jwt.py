"""Tests for the gateway's JWT issuer and verifier.

Run with: `pytest gateway/tests/test_jwt.py`
"""
import time

import pytest
from jose import jwt as jose_jwt
from jose.exceptions import ExpiredSignatureError, JWTClaimsError

from gateway.app.security.jwt import (
    KeyManager,
    issue_token,
    issue_internal_token,
    verify_token,
)


def test_keymanager_kid_is_set():
    km = KeyManager()
    assert km.kid
    assert km.kid == "ed-gateway-1"


def test_keymanager_jwk_has_required_fields():
    km = KeyManager()
    jwk = km.public_jwk()
    for k in ("kty", "kid", "use", "alg", "n", "e"):
        assert k in jwk, f"missing {k}"
    assert jwk["kty"] == "RSA"
    assert jwk["alg"] == "RS256"


def test_token_round_trip():
    token = issue_token("user-123", scopes=["rooms:read", "rooms:write"], roles=["user"], ttl_seconds=60)
    claims = verify_token(token)
    assert claims["sub"] == "user-123"
    assert claims["scopes"] == ["rooms:read", "rooms:write"]
    assert claims["roles"] == ["user"]
    assert claims["iss"] == "ed-gateway"


def test_internal_token_has_service_role():
    token = issue_internal_token("doc-service")
    claims = verify_token(token)
    assert "service" in claims["roles"]
    assert claims["sub"] == "service:doc-service"
    assert "internal" in claims["scopes"]


def test_expired_token_raises():
    token = issue_token("u", scopes=[], roles=[], ttl_seconds=-1)
    with pytest.raises(ExpiredSignatureError):
        verify_token(token)


def test_tampered_token_raises():
    token = issue_token("u", scopes=[], roles=[], ttl_seconds=60)
    # Flip a character
    parts = token.split(".")
    parts[1] = parts[1][:-2] + ("AB" if not parts[1].endswith("AB") else "CD")
    tampered = ".".join(parts)
    with pytest.raises(Exception):
        verify_token(tampered)
