import sys
import json
import urllib.request
import urllib.parse

def fetch_magma_volcano():
    """
    Fetches real-time volcanic activity from ESDM MAGMA Indonesia API.
    """
    # MAGMA provides public JSON feeds for VONA (Volcano Observatory Notice for Aviation)
    # and daily volcanic activity reports (MAGMA-VAR)
    
    url = "https://magma.esdm.go.id/api/v1/gunungapi/informasi"
    
    try:
        # Many ID gov sites require user agent
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req, timeout=10) as response:
            # We don't want to parse everything if it's huge, just verify the connection
            html = response.read().decode()
            
            # Since the open API might be protected by captcha/CORS without an API key
            # Let's see if we get JSON or HTML block
            if "{" in html[:50]:
                data = json.loads(html)
                print(json.dumps({"status": "success", "source": "MAGMA ESDM", "data_sample": "Connected"}))
            else:
                print(json.dumps({"status": "error", "message": "Received HTML/Blocked, likely requires API key or auth token."}))
                
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "message": str(e)
        }))

if __name__ == "__main__":
    fetch_magma_volcano()
