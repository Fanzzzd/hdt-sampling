"""
hdt_sampling – Python wrapper package for the native Rust extension
===================================================================

This lightweight ``__init__.py`` simply re-exports the public symbols from the
compiled extension module built by maturin/PyO3 so that

    >>> from hdt_sampling import HDTSampler

works exactly the same way as importing the extension directly while also giving
us a place to ship typing information (``__init__.pyi``) and the ``py.typed``
marker required by PEP 561.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

# The compiled extension may be named either ``_hdt_sampling`` (the modern maturin
# default when a Python source directory is present) or plain ``hdt_sampling`` when
# the crate is built directly.  We try the underscore variant first and fall back
# to the plain name so the package works in both layouts.

try:
    from ._hdt_sampling import *  # type: ignore
except ModuleNotFoundError:  # pragma: no cover
    from .hdt_sampling import *  # type: ignore  # pyright: ignore[reportMissingImports]

if TYPE_CHECKING:  # pragma: no cover
    # Re-import with full type information for static analysers
    try:
        from ._hdt_sampling import HDTSampler
    except ModuleNotFoundError:  # fallback for alternative naming scheme
        from .hdt_sampling import HDTSampler

__all__ = ["HDTSampler"]