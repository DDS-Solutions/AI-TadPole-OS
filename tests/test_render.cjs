const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();

    page.on('console', msg => {
        if (msg.type() === 'error') {
            console.error(`Browser Error: ${msg.text()}`);
        }
    });

    page.on('pageerror', error => {
        console.error(`Page Exception: ${error.message}`);
    });

    await page.goto('http://localhost:5174', { waitUntil: 'networkidle' });

    // Give it a second to render
    await page.waitForTimeout(2000);

    await browser.close();
})();
