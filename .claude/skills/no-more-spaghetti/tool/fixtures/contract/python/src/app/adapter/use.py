from app.core.name import name
import app.stray
from app.gen.made import made
import importlib
importlib.import_module(target)
try:
    import app.core.fast as impl
except ImportError:
    import app.core.slow as impl
