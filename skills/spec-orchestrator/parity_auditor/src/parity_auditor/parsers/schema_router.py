import os
import logging
from typing import List, Tuple, Optional, Dict
from .base import IParser
from .regex import RegexSchemaParser
from ..core.workspace import WorkspaceRepository

logger = logging.getLogger(__name__)

IGNORED_FILENAMES = {".gitkeep", ".DS_Store", ".gitignore"}


def _is_ignored_file(filepath: str) -> bool:
    basename = os.path.basename(filepath)
    if basename in IGNORED_FILENAMES:
        return True
    if basename.startswith("."):
        return True
    return False


class SchemaRouter(IParser):
    def __init__(self, workspace_repo: WorkspaceRepository, parsers: Optional[List[IParser]] = None):
        self.workspace_repo = workspace_repo
        if parsers is not None:
            self._parsers: List[IParser] = list(parsers)
        else:
            self._parsers: List[IParser] = [RegexSchemaParser(workspace_repo)]

    def register(self, parser: IParser, prepend: bool = False):
        if prepend:
            self._parsers.insert(0, parser)
        else:
            self._parsers.append(parser)

    def can_parse(self, filepath: str) -> bool:
        return any(parser.can_parse(filepath) for parser in self._parsers)

    def parse(self, filepath: str) -> Tuple[Optional[str], Dict[str, str]]:
        for parser in self._parsers:
            if parser.can_parse(filepath):
                return parser.parse(filepath)
        if _is_ignored_file(filepath):
            return None, {}
        ext = os.path.splitext(filepath)[1].lower()
        logger.warning(
            "Extensible schema parser not yet implemented for extension '%s' in %s",
            ext,
            os.path.basename(filepath),
        )
        return os.path.basename(filepath), {}


def parse_schema_file(
    filepath: str,
    repo: Optional[WorkspaceRepository] = None,
    router: Optional[SchemaRouter] = None,
) -> Tuple[Optional[str], Dict[str, str]]:
    if router is not None:
        return router.parse(filepath)
    if repo is None:
        repo = WorkspaceRepository()
    router = SchemaRouter(repo)
    return router.parse(filepath)

