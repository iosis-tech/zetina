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
class FieldElementsData:
    data_len: int
    data: List[int]

    def deserialize(self) -> bytes:
        FIELD_ELEMENT_CHUNK_SIZE = 31
        v = []
        for i in range(0, len(self.data) - 1):
            data = self.data[i].to_bytes(FIELD_ELEMENT_CHUNK_SIZE, "big")
            v.extend([0] * (FIELD_ELEMENT_CHUNK_SIZE - len(data)) + list(data))

        data = self.data[-1].to_bytes(FIELD_ELEMENT_CHUNK_SIZE, "big")
        data = [0] * (FIELD_ELEMENT_CHUNK_SIZE - len(data)) + list(data)
        v.extend(
            data[
                (
                    FIELD_ELEMENT_CHUNK_SIZE * len(self.data) - self.data_len
                ) : FIELD_ELEMENT_CHUNK_SIZE
            ]
        )
        return bytes(v)


@dataclasses.dataclass(frozen=True)
class JobData(Task):
    reward: int
    num_of_steps: int
    cairo_pie_compressed: FieldElementsData

    def load_task(self) -> "CairoPieTask":
        return CairoPieTask(
            cairo_pie=CairoPie.deserialize(self.cairo_pie_compressed.deserialize()),
            use_poseidon=True,
        )


@dataclasses.dataclass(frozen=True)
class Job(Task):
    job_data: JobData
    public_key: int
    signature_r: int
    signature_s: int

    def load_task(self) -> "CairoPieTask":
        return self.job_data.load_task()


@marshmallow_dataclass.dataclass(frozen=True)
class SimpleBootloaderInput(ValidatedMarshmallowDataclass):
    public_key: int
    job: Job

    fact_topologies_path: Optional[str]

    # If true, the bootloader will put all the outputs in a single page, ignoring the
    # tasks' fact topologies.
    single_page: bool
