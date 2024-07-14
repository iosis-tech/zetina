from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class DelegateRequest(_message.Message):
    __slots__ = ("cairo_pie",)
    CAIRO_PIE_FIELD_NUMBER: _ClassVar[int]
    cairo_pie: bytes
    def __init__(self, cairo_pie: _Optional[bytes] = ...) -> None: ...

class DelegateResponse(_message.Message):
    __slots__ = ("proof", "job_hash")
    PROOF_FIELD_NUMBER: _ClassVar[int]
    JOB_HASH_FIELD_NUMBER: _ClassVar[int]
    proof: bytes
    job_hash: int
    def __init__(self, proof: _Optional[bytes] = ..., job_hash: _Optional[int] = ...) -> None: ...
