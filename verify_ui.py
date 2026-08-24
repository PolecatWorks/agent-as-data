from playwright.sync_api import sync_playwright
import time

def run(playwright):
    browser = playwright.chromium.launch(headless=True)
    page = browser.new_page(viewport={"width": 1280, "height": 1024})

    # Check agents registry
    page.goto('http://localhost:4200/agent-registry')
    page.wait_for_selector('text=Agents')
    time.sleep(2)

    # Check skills registry
    page.goto('http://localhost:4200/skills-registry')
    page.wait_for_selector('text=Skills')
    time.sleep(2)

    # Find edit button for first skill and click it
    page.evaluate("""
        const cards = document.querySelectorAll('mat-card');
        for (let i = 0; i < cards.length; i++) {
            if (cards[i].innerText.includes('NetworkOptimizer')) {
                cards[i].click();
                break;
            }
        }
    """)
    time.sleep(1)

    page.evaluate("""
        const buttons = document.querySelectorAll('button');
        for (let i = 0; i < buttons.length; i++) {
            if (buttons[i].innerText.includes('Edit')) {
                buttons[i].click();
                break;
            }
        }
    """)
    time.sleep(1)

    # Click Dependencies tab
    page.evaluate("""
        const tabs = document.querySelectorAll('.mdc-tab__text-label');
        for (let i = 0; i < tabs.length; i++) {
            if (tabs[i].innerText.includes('Dependencies')) {
                tabs[i].click();
                break;
            }
        }
    """)
    time.sleep(1)

    # Scroll down to bottom of edit form
    page.evaluate("window.scrollTo(0, document.body.scrollHeight)")
    page.evaluate("""
        const editPanel = document.querySelector('.edit-skill-panel');
        if (editPanel) editPanel.scrollTo(0, editPanel.scrollHeight);
    """)
    time.sleep(1)
    page.screenshot(path='skills_uses_traits_scroll.png')

    browser.close()

with sync_playwright() as playwright:
    run(playwright)
