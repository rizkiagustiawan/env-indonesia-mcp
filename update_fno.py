with open("src/mlops/training/train_fno.py", "r") as f:
    c = f.read()
c = c.replace("from models.fno import FNO2d", "from models.fno import UFNO2d")
c = c.replace("self.model = FNO2d", "self.model = UFNO2d")
with open("src/mlops/training/train_fno.py", "w") as f:
    f.write(c)
