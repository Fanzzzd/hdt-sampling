from typing import List, Tuple, Optional, Sequence

class HDTSampler:
    """
    High-Dimensional Tree Poisson-disk sampler.

    Parameters
    ----------
    width:
        Width of the rectangular sampling domain.
    height:
        Height of the rectangular sampling domain.
    r:
        Minimum allowed Euclidean distance between any two samples.
    seed:
        Optional RNG seed.  When *None*, a random seed is drawn from the OS.

    Notes
    -----
    The heavy lifting is performed in native Rust code; this stub only
    describes the public interface for static type checkers and IDEs.
    """

    def __init__(self, width: float, height: float, r: float, seed: Optional[int] = ...) -> None: ...
    def generate(self) -> List[Tuple[float, float]]: ...