import dataclasses
from abc import abstractmethod
from dataclasses import field
from typing import ClassVar, Dict, List, Optional, Type

import marshmallow
import marshmallow.fields as mfields
import marshmallow_dataclass
from marshmallow_oneofschema import OneOfSchema

from starkware.cairo.lang.compiler.program import Program, ProgramBase, StrippedProgram
from starkware.cairo.lang.vm.cairo_pie import CairoPie
from starkware.starkware_utils.marshmallow_dataclass_fields import additional_metadata
from starkware.starkware_utils.validated_dataclass import ValidatedMarshmallowDataclass

class Task:
    @abstractmethod
    def get_program(self) -> ProgramBase:
        """
        Returns the task's Cairo program.
        """

@dataclasses.dataclass(frozen=True)
class Job(Task):
    reward: int
    num_of_steps: int
    cairo_pie: CairoPie
    registry_address: bytearray
    public_key: bytearray
    signature: bytearray

    def get_program(self) -> StrippedProgram:
        return self.cairo_pie.program

@marshmallow_dataclass.dataclass(frozen=True)
class SimpleBootloaderInput(ValidatedMarshmallowDataclass):
    identity: bytearray
    job: Job

    fact_topologies_path: Optional[str]

    # If true, the bootloader will put all the outputs in a single page, ignoring the
    # tasks' fact topologies.
    single_page: bool