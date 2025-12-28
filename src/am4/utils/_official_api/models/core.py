from __future__ import annotations

from pydantic import BaseModel, Field, field_validator


class Status(BaseModel):
    success: bool = Field(alias="request", default=False)
    requests_remaining: int | None = None
    description: str | None = None

    @field_validator("success", mode="before")
    @classmethod
    def request_to_bool(cls, v):
        if isinstance(v, str):
            return v == "success"
        return v
