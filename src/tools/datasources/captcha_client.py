#!/usr/bin/env python3
"""CAPTCHA Client — thin wrapper for Boterdrop-Solver API.
Supports: reCAPTCHA v3, Turnstile, cf_clearance, AWS WAF.
Fallback: inline Playwright if solver not running.
"""
import time
import requests

SOLVER_URL = "http://127.0.0.1:8000"
POLL_INTERVAL = 2
MAX_POLLS = 45  # 90 seconds max


def _solver_available():
    """Check if Boterdrop-Solver is running."""
    try:
        requests.get(f"{SOLVER_URL}/", timeout=2)
        return True
    except:
        return False


def solve_recaptcha_v3(url, sitekey, action="submit", timeout_s=90):
    """Solve reCAPTCHA v3 via Boterdrop-Solver API.
    Returns token string or None on failure.
    """
    if not _solver_available():
        print("WARNING: Boterdrop-Solver tidak berjalan di localhost:8000")
        print("Fallback ke inline Playwright...")
        return _fallback_recaptcha(url, sitekey, action)

    try:
        # Submit task
        resp = requests.get(f"{SOLVER_URL}/recaptchaV3", params={
            "url": url, "sitekey": sitekey, "action": action
        }, timeout=10)
        data = resp.json()
        task_id = data.get("task_id")
        if not task_id:
            print(f"ERROR: No task_id — {data}")
            return None

        # Poll for result
        max_polls = int(timeout_s / POLL_INTERVAL)
        for i in range(max_polls):
            time.sleep(POLL_INTERVAL)
            poll = requests.get(f"{SOLVER_URL}/result", params={"id": task_id}, timeout=5)
            result = poll.json()
            status = result.get("status")

            if status == "success":
                token = result.get("value")
                elapsed = result.get("elapsed_time", 0)
                print(f"reCAPTCHA v3 solved in {elapsed:.1f}s")
                return token
            elif status == "error":
                print(f"ERROR: Solver gagal — {result}")
                return None

        print(f"TIMEOUT: reCAPTCHA v3 tidak terselesaikan dalam {timeout_s}s")
        return None

    except Exception as e:
        print(f"ERROR: {e}")
        return _fallback_recaptcha(url, sitekey, action)


def solve_turnstile(url, sitekey, timeout_s=60):
    """Solve Cloudflare Turnstile via Boterdrop-Solver API."""
    if not _solver_available():
        print("WARNING: Boterdrop-Solver tidak berjalan")
        return None

    try:
        resp = requests.get(f"{SOLVER_URL}/turnstile", params={
            "url": url, "sitekey": sitekey
        }, timeout=10)
        task_id = resp.json().get("task_id")
        if not task_id:
            return None

        max_polls = int(timeout_s / POLL_INTERVAL)
        for i in range(max_polls):
            time.sleep(POLL_INTERVAL)
            poll = requests.get(f"{SOLVER_URL}/result", params={"id": task_id}, timeout=5)
            result = poll.json()
            if result.get("status") == "success":
                print(f"Turnstile solved in {result.get('elapsed_time', 0):.1f}s")
                return result.get("value")
            elif result.get("status") == "error":
                return None
        return None
    except Exception as e:
        print(f"ERROR: {e}")
        return None


def _fallback_recaptcha(url, sitekey, action="submit"):
    """Fallback: solve reCAPTCHA v3 via inline Playwright."""
    try:
        import asyncio
        from playwright.async_api import async_playwright

        async def _solve():
            async with async_playwright() as p:
                browser = await p.chromium.launch(
                    headless=True,
                    args=['--disable-blink-features=AutomationControlled', '--no-sandbox']
                )
                context = await browser.new_context(
                    user_agent='Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36'
                )
                page = await context.new_page()
                await page.add_init_script("Object.defineProperty(navigator, 'webdriver', {get: () => undefined})")

                await page.goto(url, timeout=30000)

                # Wait for grecaptcha
                for _ in range(15):
                    await page.wait_for_timeout(2000)
                    ready = await page.evaluate(
                        "() => typeof window.grecaptcha !== 'undefined' && typeof window.grecaptcha.execute === 'function'"
                    )
                    if ready:
                        break

                if not ready:
                    await browser.close()
                    return None

                token = await page.evaluate(
                    f"async () => await window.grecaptcha.execute('{sitekey}', {{action: '{action}'}})"
                )
                await browser.close()
                return token if token and len(token) > 20 else None

        return asyncio.run(_solve())
    except Exception as e:
        print(f"Fallback failed: {e}")
        return None


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 4:
        print("Usage: captcha_client.py recaptcha <url> <sitekey> [action]")
        print("       captcha_client.py turnstile <url> <sitekey>")
        sys.exit(1)

    mode = sys.argv[1]
    if mode == "recaptcha":
        action = sys.argv[4] if len(sys.argv) > 4 else "submit"
        token = solve_recaptcha_v3(sys.argv[2], sys.argv[3], action)
        if token:
            print(f"TOKEN: {token[:80]}...")
        else:
            print("FAILED")
    elif mode == "turnstile":
        token = solve_turnstile(sys.argv[2], sys.argv[3])
        if token:
            print(f"TOKEN: {token[:80]}...")
        else:
            print("FAILED")
