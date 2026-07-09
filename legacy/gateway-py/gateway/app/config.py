from pydantic_settings import BaseSettings, SettingsConfigDict
class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")
    DATABASE_URL: str = "postgres://ed:ed@postgres:5432/ed"
    MONGO_URL:    str = "mongodb://mongo:27017/ed"
    REDIS_URL:    str = "redis://redis:6379"
    RABBITMQ_URL: str = "amqp://guest:guest@rabbit:5672/"
    JWT_ISSUER:   str = "ed-gateway"
    JWT_AUDIENCE: str = "ed-services"
    JWKS_URL:     str = "http://localhost:8080/.well-known/jwks.json"
    INTERNAL_SERVICE_TOKEN_SECRET: str = "changeme"
    OTEL_EXPORTER_OTLP_ENDPOINT: str = ""
    GATEWAY_HOST: str = "0.0.0.0"
    GATEWAY_PORT: int = 8080
    SERVICE_NAME: str = "gateway"
    SERVICES: dict = {
        "room-service":  {"base_url": "http://room-service:8080"},
        "doc-service":   {"base_url": "http://doc-service:8080"},
        "latex-service": {"base_url": "http://latex-service:8080"},
    }
settings = Settings()
