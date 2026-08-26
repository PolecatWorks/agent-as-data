#!/bin/bash
set -ex

# Use Garden's provided namespace if available, otherwise fallback to the PR pattern
NS="${GARDEN_NAMESPACE:-agent-as-data-pr-${PR_NUMBER:-local}}"

echo "Using namespace: $NS"

POD_NAME="robot-test-runner"

# Ensure kubectl is in the path
export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"

# Wait for the pod to be ready
kubectl wait --for=condition=Ready pod/$POD_NAME -n $NS --timeout=300s

# Install required library synchronously
echo "Installing robotframework-requests..."
kubectl exec $POD_NAME -n $NS -- /bin/bash -c "pip install robotframework-requests"
echo "pip install finished."

# Extract backend pod IP
BE_POD_IP=$(kubectl get pods -l app=agent-as-data -n $NS -o jsonpath='{.items[0].status.podIP}')
echo "Backend Pod IP: $BE_POD_IP"

# Create target directory and copy tests
kubectl exec $POD_NAME -n $NS -- rm -rf /tmp/robot-tests /tmp/reports
kubectl exec $POD_NAME -n $NS -- mkdir -p /tmp/robot-tests /tmp/reports
kubectl cp ./integration-tests/tests $POD_NAME:/tmp/robot-tests -n $NS
kubectl cp ./integration-tests/lib $POD_NAME:/tmp/lib -n $NS

# Execute tests
TEST_EXIT_CODE=0
kubectl exec $POD_NAME -n $NS -- /bin/bash -c "
  export HOME=/home/pwuser
  cd /tmp
  export PATH=\$PATH:/home/pwuser/.local/bin:/home/pwuser/.venv/bin

  BE_BASE_URL=\"http://agent-as-data:8080\"
  FE_BASE_URL=\"http://agent-as-data-fe:80\"

  robot --variable BE_POD_IP:$BE_POD_IP --variable BE_BASE_URL:\$BE_BASE_URL --variable FE_BASE_URL:\$FE_BASE_URL --loglevel DEBUG -d /tmp/reports /tmp/robot-tests
" || TEST_EXIT_CODE=$?

# Pull reports back
echo "Pulling reports back from the robot runner..."
rm -rf ./integration-tests/reports
mkdir -p ./integration-tests/reports
kubectl cp $NS/$POD_NAME:/tmp/reports ./integration-tests/reports || echo "Failed to copy reports"

exit $TEST_EXIT_CODE
