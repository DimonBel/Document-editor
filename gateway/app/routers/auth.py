from fastapi import APIRouter
from pydantic import BaseModel
from gateway.app.security.jwt import issue_token, issue_internal_token, key_manager
router = APIRouter(prefix="/auth")
class LoginIn(BaseModel):
    username: str; password: str
class TokenOut(BaseModel):
    access_token: str; token_type: str = "Bearer"; expires_in: int
@router.post("/login", response_model=TokenOut)
async def login(body: LoginIn):
    # Stub auth: any password works in dev.
    return TokenOut(access_token=issue_token(body.username, scopes=["rooms:read", "rooms:write"], roles=["user"]), expires_in=900)
@router.post("/internal", response_model=TokenOut)
async def internal(body: dict):
    return TokenOut(access_token=issue_internal_token(body.get("service", "gateway")), expires_in=60)
@router.get("/.well-known/jwks.json")
async def jwks():
    return {"keys": [key_manager.public_jwk()]}
