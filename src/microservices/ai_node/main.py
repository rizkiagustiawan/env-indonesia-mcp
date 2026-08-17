import torch
import numpy as np

def health_check():
    return {
        "status": "ok",
        "service": "ai-inference-node",
        "device": "cuda" if torch.cuda.is_available() else "cpu",
        "pytorch_version": torch.__version__
    }

if __name__ == "__main__":
    print(health_check())
