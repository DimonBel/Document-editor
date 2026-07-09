from jose import jwt, jwk
from jose.utils import long_to_base64
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from gateway.app.config import settings
from datetime import datetime, timedelta, timezone
import json

class KeyManager:
    def __init__(self):
        self._key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
        self._kid = "ed-gateway-1"
    @property
    def kid(self) -> str: return self._kid
    def private_pem(self) -> bytes:
        return self._key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption())
    def public_jwk(self) -> dict:
        nums = self._key.public_key().public_numbers()
        return {"kty": "RSA", "kid": self._kid, "use": "sig", "alg": "RS256",
                "n": long_to_base64(nums.n).decode(), "e": long_to_base64(nums.e).decode()}
    def sign(self, claims: dict) -> str:
        return jwt.encode(claims, self.private_pem().decode(), algorithm="RS256", kid=self._kid, headers={"kid": self._kid})
key_manager = KeyManager()
def issue_token(subject: str, scopes: list[str], roles: list[str], ttl_seconds: int = 900) -> str:
    now = datetime.now(timezone.utc)
    claims = {"iss": settings.JWT_ISSUER, "aud": settings.JWT_AUDIENCE, "sub": subject,
              "iat": int(now.timestamp()), "exp": int((now + timedelta(seconds=ttl_seconds)).timestamp()),
              "scopes": scopes, "roles": roles}
    return key_manager.sign(claims)
def issue_internal_token(service: str, ttl_seconds: int = 60) -> str:
    now = datetime.now(timezone.utc)
    claims = {"iss": settings.JWT_ISSUER, "aud": "internal", "sub": f"service:{service}",
              "iat": int(now.timestamp()), "exp": int((now + timedelta(seconds=ttl_seconds)).timestamp()),
              "scopes": ["internal"], "roles": ["service"]}
    return key_manager.sign(claims)
