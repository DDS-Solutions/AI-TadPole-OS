"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[debug_sector_fault]` in system logs.
"""


import asyncio
from playwright.async_api import async_playwright

async def run():
    async with async_playwright() as p:
        # Launch browser
        browser = await p.chromium.launch()
        context = await browser.new_context()
        page = await context.new_page()

        # Monitor console logs
        page.on("console", lambda msg: print(f"CONSOLE: {msg.type}: {msg.text}"))
        page.on("pageerror", lambda err: print(f"PAGE ERROR: {err}"))

        print("Navigating to dashboard...")
        try:
            await page.goto("http://localhost:8000/dashboard", timeout=30000)
            print("Page loaded. Waiting for errors...")
            await asyncio.sleep(10) # Wait for renders/handshakes
            
            # Check for Sector Fault banner
            fault_visible = await page.is_visible("text=Neural Sector Fault")
            if fault_visible:
                print("Confirmed: Neural Sector Fault is visible on screen.")
            
            # Capture screenshot for visual context
            await page.screenshot(path="D:/TadpoleOS-Dev/data/sector_fault_debug.png")
            print("Screenshot saved to D:/TadpoleOS-Dev/data/sector_fault_debug.png")

        except Exception as e:
            print(f"Error during navigation: {e}")
        
        await browser.close()

if __name__ == "__main__":
    asyncio.run(run())

# Metadata: [debug_sector_fault]
