from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel, Field

from .core import Status


class Alliance(BaseModel):
    id: int | None = 0
    name: str
    rank: int
    member_count: int = Field(alias="members")
    max_members: int = Field(alias="maxMembers")
    value: int  # broken!
    ipo: bool
    min_sv: float = Field(alias="minSV")


class Member(BaseModel):
    id: int | None = 0
    username: str = Field(alias="company")
    joined: datetime
    flights: int
    contributed: int
    daily_contribution: int = Field(alias="dailyContribution")
    online: datetime
    sv: float = Field(alias="shareValue")
    season: int | None = 0  # none if alliance not participating in season


class AllianceResponse(BaseModel):
    status: Status
    alliance: list[Alliance]
    members: list[Member]
