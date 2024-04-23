import dataclasses
from abc import abstractmethod
from typing import Optional

import marshmallow_dataclass

from starkware.cairo.lang.compiler.program import ProgramBase, StrippedProgram
from starkware.cairo.lang.vm.cairo_pie import CairoPie
from starkware.starkware_utils.validated_dataclass import ValidatedMarshmallowDataclass


class TaskSpec(ValidatedMarshmallowDataclass):
    """
    Contains task's specification.
    """

    @abstractmethod
    def load_task(self, memory=None, args_start=None, args_len=None) -> "Task":
        """
        Returns the corresponding task.
        """


class Task:
    @abstractmethod
    def get_program(self) -> ProgramBase:
        """
        Returns the task's Cairo program.
        """


@dataclasses.dataclass(frozen=True)
class CairoPieTask(Task):
    cairo_pie: CairoPie
    use_poseidon: bool

    def get_program(self) -> StrippedProgram:
        return self.cairo_pie.program


@dataclasses.dataclass(frozen=True)
class Job(Task):
    reward: int
    num_of_steps: int
    cairo_pie: bytearray
    registry_address: bytearray
    public_key: bytearray
    signature: bytearray

    def load_task(self) -> "CairoPieTask":
        """
        Loads the PIE to memory.
        """
        return CairoPieTask(
            cairo_pie=CairoPie.deserialize(self.cairo_pie),
            use_poseidon=self.use_poseidon,
        )


@marshmallow_dataclass.dataclass(frozen=True)
class SimpleBootloaderInput(ValidatedMarshmallowDataclass):
    identity: bytearray
    job: Job

    fact_topologies_path: Optional[str]

    # If true, the bootloader will put all the outputs in a single page, ignoring the
    # tasks' fact topologies.
    single_page: bool
