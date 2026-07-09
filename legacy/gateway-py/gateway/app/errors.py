from fastapi import Request
from fastapi.responses import JSONResponse
class AppError(Exception):
    def __init__(self, status: int, title: str, detail: str | None = None, type_suffix: str | None = None):
        self.status = status; self.title = title; self.detail = detail; self.type_suffix = type_suffix
async def app_error_handler(_: Request, exc: AppError) -> JSONResponse:
    return JSONResponse(status_code=exc.status, content={
        "type":   f"about:blank#{exc.status}" if not exc.type_suffix else f"https://docs.example/errors/{exc.type_suffix}",
        "title":  exc.title,
        "status": exc.status,
        "detail": exc.detail,
    }, media_type="application/problem+json")
