from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel, Field, field_validator

from .core import Status


class User(BaseModel):
    id: int | None = 0
    username: str = Field(alias="company")
    level: int
    online: bool
    share: float
    shares_available: int
    shares_sold: int
    ipo: int
    fleet_count: int = Field(alias="fleet")
    routes: int
    alliance: str
    achievements: int
    game_mode: bool
    rank: int
    reputation: int
    cargo_reputation: int
    founded: datetime
    logo: str

    @field_validator("game_mode", mode="before")
    @classmethod
    def game_mode_to_bool(cls, v: str | bool):
        return v == "Realism" if isinstance(v, str) else v


class Share(BaseModel):
    ts: datetime = Field(alias="date")
    share: float


class Award(BaseModel):
    ts: datetime = Field(alias="awarded")
    award: str


class AircraftCount(BaseModel):
    aircraft: str
    amount: int


class RouteDetail(BaseModel):
    origin: str = Field(alias="dep")
    stopover: str = Field(alias="stop", default="")
    destination: str = Field(alias="arrival")
    distance: int
    arrived: datetime

    @field_validator("stopover", mode="before")
    @classmethod
    def stopover_null_to_empty(cls, v: str | None):
        return "" if v is None else v


class UserResponse(BaseModel):
    status: Status
    user: User
    share_log: list[Share] = Field(alias="share_development")
    awards: list[Award]
    fleet: list[AircraftCount]
    route_list: list[RouteDetail] = Field(alias="routes")
