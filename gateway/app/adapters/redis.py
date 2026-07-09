import redis.asyncio as aioredis
from gateway.app.config import settings
_client: aioredis.Redis | None = None
async def connect(url: str):
    global _client
    _client = aioredis.from_url(url, decode_responses=True)
async def close():
    if _client: await _client.close()
def client() -> aioredis.Redis:
    assert _client is not None, "redis not connected"
    return _client
async def try_acquire_rate_limit(key: str, capacity: int, refill_per_sec: int) -> str:
    import time
    bucket = int(time.time()) // max(refill_per_sec, 1)
    full_key = f"rl:{key}:{bucket}"
    c = client()
    n = await c.incr(full_key)
    if n == 1:
        await c.expire(full_key, 60)
    return "deny" if n > capacity else "allow"
