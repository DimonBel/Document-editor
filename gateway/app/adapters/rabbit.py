import aio_pika, asyncio, json, logging
from gateway.app.config import settings
log = logging.getLogger(__name__)
_connection: aio_pika.abc.AbstractRobustConnection | None = None
_channel: aio_pika.abc.AbstractChannel | None = None
_rooms_subscribers: set[asyncio.Queue] = set()
async def connect(url: str):
    global _connection, _channel
    _connection = await aio_pika.connect_robust(url)
    _channel = await _connection.channel()
    await _channel.declare_exchange("ed.events", aio_pika.ExchangeType.TOPIC, durable=True)
async def close():
    global _connection
    if _connection: await _connection.close()
async def publish(topic: str, body: dict):
    assert _channel is not None, "rabbit not connected"
    msg = aio_pika.Message(body=json.dumps(body).encode("utf-8"), content_type="application/json")
    await _channel.default_exchange.publish(msg, routing_key=topic)
async def subscribe_room_events():
    assert _channel is not None
    queue = await _channel.declare_queue("ed.realtime-gateway", durable=False, auto_delete=True)
    await queue.bind(_channel.default_exchange, routing_key="room.*")
    async with queue.iterator() as it:
        async for msg in it:
            async with msg.process():
                data = json.loads(msg.body)
                for q in list(_rooms_subscribers):
                    await q.put(data)
async def register_sse_consumer() -> asyncio.Queue:
    q: asyncio.Queue = asyncio.Queue()
    _rooms_subscribers.add(q)
    return q
async def unregister_sse_consumer(q: asyncio.Queue):
    _rooms_subscribers.discard(q)
