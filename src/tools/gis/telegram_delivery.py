#!/usr/bin/env python3
"""Auto-Delivery Media to Telegram
Membaca Chat ID dari config ZeroClaw atau menggunakan default.
"""
import sys, os, requests, tomli

def get_chat_id():
    try:
        with open(os.path.expanduser('~/.zeroclaw/config.toml'), "rb") as f:
            c = tomli.load(f)
        return c['peer_groups']['telegram_default']['external_peers'][0]
    except:
        return "775545807"

def send_to_telegram(file_path, caption):
    bot_token = "8802330994:AAGvzwFZzFCMzMtdxn36Dq2R1mawkCwvtZA"
    chat_id = get_chat_id()
    
    if not os.path.exists(file_path):
        return f"File {file_path} tidak ditemukan."
        
    ext = os.path.splitext(file_path)[1].lower()
    
    if ext in ['.png', '.jpg']:
        url = f"https://api.telegram.org/bot{bot_token}/sendPhoto"
        files = {'photo': open(file_path, 'rb')}
    elif ext == '.gif':
        url = f"https://api.telegram.org/bot{bot_token}/sendAnimation"
        files = {'animation': open(file_path, 'rb')}
    else:
        url = f"https://api.telegram.org/bot{bot_token}/sendDocument"
        files = {'document': open(file_path, 'rb')}
        
    data = {'chat_id': chat_id, 'caption': caption, 'parse_mode': 'HTML'}
    
    try:
        resp = requests.post(url, data=data, files=files, timeout=60)
        resp.raise_for_status()
        return "Terkirim ke Telegram!"
    except Exception as e:
        return f"Gagal mengirim: {e}"

if __name__ == "__main__":
    if len(sys.argv) > 1:
        print(send_to_telegram(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else ""))
