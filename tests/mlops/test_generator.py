import os
import pytest
import sys
from unittest.mock import patch, MagicMock

# Mock torch so we don't need it installed to pass the plan test
sys.modules['torch'] = MagicMock()
import src.mlops.data_generator.generator
src.mlops.data_generator.generator.torch = MagicMock()

from src.mlops.data_generator.generator import generate_batch

@pytest.mark.asyncio
async def test_generate_batch_creates_files(tmp_path):
    output_dir = tmp_path / ".data"
    
    # We mock the torch.save so it actually touches the file we expect
    def mock_save(obj, path):
        with open(path, 'w') as f:
            f.write("mock")
            
    src.mlops.data_generator.generator.torch.save = mock_save
    src.mlops.data_generator.generator.torch.tensor.return_value.reshape.return_value = "mock_tensor"
    
    await generate_batch(num_samples=2, output_dir=str(output_dir), mock=True)
    
    assert os.path.exists(output_dir)
    files = os.listdir(output_dir)
    assert len(files) == 2
    assert files[0].endswith(".pt")
