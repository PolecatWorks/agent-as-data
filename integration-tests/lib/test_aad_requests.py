import os
import unittest
from unittest.mock import patch, MagicMock

from AADRequests import AADRequests

class TestAADRequestsTelemetry(unittest.TestCase):
    def setUp(self):
        self.req = AADRequests()
        # Clean environment overrides
        for var in ["HAMS_BE_URL", "HAMS_MCP_URL", "BE_POD_IP", "MCP_POD_IP", "KUBERNETES_SERVICE_HOST"]:
            if var in os.environ:
                del os.environ[var]

    def test_default_local_endpoints(self):
        self.assertEqual(self.req.resolve_hams_endpoint("backend"), "http://localhost:8079")
        self.assertEqual(self.req.resolve_hams_endpoint("be"), "http://localhost:8079")
        self.assertEqual(self.req.resolve_hams_endpoint("mcp"), "http://localhost:8078")
        self.assertEqual(self.req.resolve_hams_endpoint("aad-mcp-container"), "http://localhost:8078")

    def test_explicit_url_overrides(self):
        with patch.dict(os.environ, {"HAMS_BE_URL": "http://custom-be:9999", "HAMS_MCP_URL": "http://custom-mcp:8888"}):
            self.assertEqual(self.req.resolve_hams_endpoint("backend"), "http://custom-be:9999")
            self.assertEqual(self.req.resolve_hams_endpoint("mcp"), "http://custom-mcp:8888")

    def test_pod_ip_addressing(self):
        with patch.dict(os.environ, {"BE_POD_IP": "10.244.1.20", "MCP_POD_IP": "10.244.2.30"}):
            self.assertEqual(self.req.resolve_hams_endpoint("backend"), "http://10.244.1.20:8079")
            self.assertEqual(self.req.resolve_hams_endpoint("mcp"), "http://10.244.2.30:8079")

    def test_explicit_url_takes_precedence_over_pod_ip(self):
        with patch.dict(os.environ, {"HAMS_BE_URL": "http://override:8079", "BE_POD_IP": "10.244.1.20"}):
            self.assertEqual(self.req.resolve_hams_endpoint("backend"), "http://override:8079")

    def test_invalid_service_raises_error(self):
        with self.assertRaises(ValueError):
            self.req.resolve_hams_endpoint("invalid_service")

    @patch("requests.get")
    def test_get_hams_metrics(self, mock_get):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = "app_info{name=\"aad-be-container\"} 1\ntokio_workers_count 4"
        mock_get.return_value = mock_response

        metrics = self.req.get_hams_metrics("backend")
        mock_get.assert_called_once_with("http://localhost:8079/hams/metrics", timeout=5)
        self.assertIn("tokio_workers_count", metrics)

    @patch("requests.get")
    def test_get_hams_health_ready(self, mock_get):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_get.return_value = mock_response

        healthy = self.req.get_hams_health("backend", endpoint="ready")
        mock_get.assert_called_once_with("http://localhost:8079/hams/ready", timeout=5)
        self.assertTrue(healthy)

if __name__ == "__main__":
    unittest.main()
