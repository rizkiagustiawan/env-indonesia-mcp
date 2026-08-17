import pytest
import sys
from unittest.mock import MagicMock

sys.modules['torch'] = MagicMock()
sys.modules['torch.nn'] = MagicMock()
sys.modules['torch.nn.functional'] = MagicMock()

def test_ufno_exists():
    try:
        from src.microservices.ai_node.models.fno import UFNO2d
        assert True
    except ImportError:
        pytest.fail("UFNO2d not implemented")
