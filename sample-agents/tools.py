from typing import Dict, Any, List

def run_ocr(image_path: str) -> Dict[str, Any]:
    """
    Simulates running OCR on a receipt image.
    In a real implementation, this would call Tesseract or Google Document AI.
    """
    print(f"Running OCR on {image_path}...")
    return {
        "raw_text": "MERCHANT: ACME Corp\nDATE: 2023-10-25\nTOTAL: 150.00 EUR",
        "confidence": 0.95
    }

def convert_currency(amount: float, from_currency: str, to_currency: str) -> float:
    """
    Simulates converting currency.
    In a real implementation, this would call an API like Fixer or ExchangeRate-API.
    """
    print(f"Converting {amount} {from_currency} to {to_currency}...")
    # Mock exchange rate
    rate = 1.1 if from_currency == "EUR" and to_currency == "USD" else 1.0
    return amount * rate

def lookup_hr_org_chart(submitter_id: str) -> Dict[str, str]:
    """
    Simulates looking up the HR org chart to find a manager.
    In a real implementation, this would call Workday or BambooHR APIs.
    """
    print(f"Looking up manager for employee {submitter_id}...")
    # Mock lookup
    return {
        "manager_id": "mgr_8899",
        "manager_name": "Jane Doe",
        "manager_email": "jane.doe@example.com"
    }

def lookup_cost_center(cost_center_id: str) -> Dict[str, Any]:
    """
    Simulates looking up cost center details.
    """
    print(f"Looking up cost center {cost_center_id}...")
    return {
        "department": "Engineering",
        "budget_remaining": 50000.00,
        "requires_vp_approval": False
    }

def dispatch_slack_notification(user_email: str, message: str) -> bool:
    """
    Simulates sending a Slack message.
    """
    print(f"Sending Slack message to {user_email}:\n{message}")
    return True
