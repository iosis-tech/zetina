import dataclasses
from abc import abstractmethod
from typing import List, Optional
import marshmallow_dataclass
from starkware.cairo.lang.compiler.program import ProgramBase, StrippedProgram
from starkware.cairo.lang.vm.cairo_pie import CairoPie
from starkware.starkware_utils.validated_dataclass import ValidatedMarshmallowDataclass


class TaskSpec(ValidatedMarshmallowDataclass):
    @abstractmethod
    def load_task(self) -> "Task":
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
class JobData(Task):
    reward: int
    num_of_steps: int
    cairo_pie_compressed: List[int]
    registry_address: str

    def load_task(self) -> "CairoPieTask":
        return CairoPieTask(
            cairo_pie=CairoPie.deserialize(bytes(self.cairo_pie_compressed)),
            use_poseidon=True,
        )


@dataclasses.dataclass(frozen=True)
class Job(Task):
    job_data: JobData
    public_key: List[int]
    signature: List[int]

    def load_task(self) -> "CairoPieTask":
        return self.job_data.load_task()


@marshmallow_dataclass.dataclass(frozen=True)
class SimpleBootloaderInput(ValidatedMarshmallowDataclass):
    identity: str
    job: Job

    fact_topologies_path: Optional[str]

    # If true, the bootloader will put all the outputs in a single page, ignoring the
    # tasks' fact topologies.
    single_page: bool
