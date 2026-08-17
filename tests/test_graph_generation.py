import os
import subprocess

def test_graphs_generated():
    assert os.path.exists("docs/architecture_internal.svg")
    assert os.path.exists("docs/architecture_external.svg")
