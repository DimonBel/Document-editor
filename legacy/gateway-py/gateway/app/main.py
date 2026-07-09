from fastapi import FastAPI
from gateway.app.routers import health, api, ws, auth, realtime
from gateway.app.config import settings
from gateway.app.adapters import rabbit, redis, mongo
from gateway.app.errors import app_error_handler, AppError
from contextlib import asynccontextmanager
import logging

log = logging.getLogger(__name__)

@asynccontextmanager
async def lifespan(app: FastAPI):
    log.info("starting gateway")
    await redis.connect(settings.REDIS_URL)
    await rabbit.connect(settings.RABBITMQ_URL)
    await mongo.connect(settings.MONGO_URL, "ed")
    await rabbit.subscribe_room_events()
    yield
    log.info("stopping gateway")
    await rabbit.close()
    await redis.close()
    await mongo.close()

app = FastAPI(title="ed-gateway", version="0.1.0", lifespan=lifespan)
app.add_exception_handler(AppError, app_error_handler)
app.include_router(health.router)
app.include_router(auth.router)
app.include_router(api.router)
app.include_router(ws.router)
app.include_router(realtime.router)
