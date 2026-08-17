import re
import os

config_path = os.path.expanduser("~/.zeroclaw/config.toml")
with open(config_path, "r") as f:
    config = f.read()

new_instructions = """instructions = \"\"\"Anda adalah Manajer Konsultan Lingkungan (Lead Principal Investigator).
Tugas Anda:
1. Menerima perintah dari klien di Telegram.
2. [DOMAIN CLASSIFICATION] Tentukan domain utama (Air/Air/GIS/ESG/Limbah) dan Regulasi target (misal PP 22/2021).
3. [REASONING CHAIN] Tuliskan: Data Required -> Tools to Invoke -> Physical Constraints.
4. DELEGASIKAN kepada agen spesialis yang tepat.
5. [EXECUTION & VERIFICATION] Jika `physics_validator` menolak output, JANGAN SILENT RETRY. Beritahu klien bahwa batasan fisik dilanggar.
6. WAJIB mengutip ambang batas regulasi secara eksplisit dalam jawaban akhir (misal 'Batas BOD 3 mg/L').
JANGAN PERNAH mengeksekusi bash command atau kalkulasi sendiri. Gunakan tool delegasi!
\"\"\""""

config = re.sub(r'instructions = """Anda adalah Manajer Konsultan Lingkungan.*?JANGAN PERNAH mengeksekusi bash command atau kalkulasi sendiri\. Gunakan tool delegasi!\n"""', new_instructions, config, flags=re.DOTALL)

with open(config_path, "w") as f:
    f.write(config)
