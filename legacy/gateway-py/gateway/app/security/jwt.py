"""Security module for the gateway.

Exposes:
- KeyManager   -- RSA keypair + JWKS exposure
- issue_token  -- short-lived user JWT
- issue_internal_token -- service-to-service token
"""
from jose import jwt
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from jose.utils import long_to_base64
from gateway.app.config import settings
from datetime import datetime, timedelta, timezone


class KeyManager:
    """Holds a single RSA keypair for the lifetime of the process.

    The keypair is generated at startup. In production you'd load it from
    disk or a KMS, with a stable `kid`.
    """

    def __init__(self):
        self._key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
        self._kid = "ed-gateway-1"

    @property
    def kid(self) -> str:
        return self._kid

    def private_pem(self) -> bytes:
        return self._key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption(),
        )

    def public_jwk(self) -> dict:
        nums = self._key.public_key().public_numbers()
        return {
            "kty": "RSA",
            "kid": self._kid,
            "use": "sig",
            "alg": "RS256",
            "n": long_to_base64(nums.n).decode(),
            "e": long_to_base64(nums.e).decode(),
        }

    def sign(self, claims: dict) -> str:
        return jwt.encode(
            claims,
            self.private_pem().decode(),
            algorithm="RS256",
            headers={"kid": self._kid},
        )


key_manager = KeyManager()


def issue_token(subject: str, scopes: list[str], roles: list[str], ttl_seconds: int = 900) -> str:
    """Issue a short-lived (default 15 min) RS256 user JWT."""
    now = datetime.now(timezone.utc)
    claims = {
        "iss": settings.JWT_ISSUER,
        "aud": settings.JWT_AUDIENCE,
        "sub": subject,
        "iat": int(now.timestamp()),
        "exp": int((now + timedelta(seconds=ttl_seconds)).timestamp()),
        "scopes": scopes,
        "roles": roles,
    }
    return key_manager.sign(claims)


def issue_internal_token(service: str, ttl_seconds: int = 60) -> str:
    """Issue a short-lived (default 60 sec) service-to-service token."""
    now = datetime.now(timezone.utc)
    claims = {
        "iss": settings.JWT_ISSUER,
        "aud": "internal",
        "sub": f"service:{service}",
        "iat": int(now.timestamp()),
        "exp": int((now + timedelta(seconds=ttl_seconds)).timestamp()),
        "scopes": ["internal"],
        "roles": ["service"],
    }
    return key_manager.sign(claims)


def verify_token(token: str) -> dict:
    """Verify an RS256 token issued by this gateway.

    Raises `jose.JWTError` on failure.
    """
    return jwt.decode(
        token,
        key_manager.private_pem().decode(),  # jose accepts the private key
        algorithms=["RS256"],
        audience=settings.JWT_AUDIENCE,
        issuer=settings.JWT_ISSUER,
    )
