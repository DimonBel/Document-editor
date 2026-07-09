"""Tests for the rate-limiter adapter (uses fakeredis)."""
import pytest

from gateway.app.adapters import redis as redis_adapter


@pytest.mark.asyncio
async def test_rate_limit_allows_within_capacity(monkeypatch):
    # fakeredis is an in-memory async redis replacement
    fakeredis = pytest.importorskip("fakeredis.aioredis")
    fake = fakeredis.FakeRedis(decode_responses=True)
    monkeypatch.setattr(redis_adapter, "_client", fake)

    # Try 5 acquires with capacity=10 -- all should pass
    for _ in range(5):
        d = await redis_adapter.try_acquire_rate_limit("user-1", capacity=10, refill_per_sec=60)
        assert d == "allow"


@pytest.mark.asyncio
async def test_rate_limit_denies_over_capacity(monkeypatch):
    fakeredis = pytest.importorskip("fakeredis.aioredis")
    fake = fakeredis.FakeRedis(decode_responses=True)
    monkeypatch.setattr(redis_adapter, "_client", fake)

    # Try 15 acquires with capacity=10 -- last 5 should deny
    decisions = []
    for _ in range(15):
        d = await redis_adapter.try_acquire_rate_limit("user-2", capacity=10, refill_per_sec=60)
        decisions.append(d)
    assert "deny" in decisions
    assert decisions.count("deny") >= 5
