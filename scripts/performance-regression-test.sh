#!/bin/bash
# Performance Regression Test - Monitors parallel test execution performance
# Alerts on >10% degradation from baseline metrics

set -euo pipefail

# Configuration
BASELINE_FILE=".performance-baseline.json"
CURRENT_RUN_FILE=".performance-current.json"
REGRESSION_THRESHOLD=10  # Percentage degradation to trigger alert
COMPARE_ONLY=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --compare-only)
      COMPARE_ONLY=true
      shift
      ;;
    --baseline)
      BASELINE_FILE="$2"
      shift 2
      ;;
    --current)
      CURRENT_RUN_FILE="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--compare-only] [--baseline FILE] [--current FILE]"
      exit 1
      ;;
  esac
done

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}${BOLD}ccswarm Performance Regression Test${NC}"
echo ""

# Function to capture current performance metrics
capture_metrics() {
    local output_file="$1"

    echo -e "${YELLOW}Running parallel test suite to capture metrics...${NC}"

    local start_time=$(date +%s)

    # Run parallel tests and capture results
    if ~/Projects/ccswarm/scripts/parallel_test_runner.sh > /tmp/perf-test-run.log 2>&1; then
        local end_time=$(date +%s)
        local total_duration=$((end_time - start_time))

        # Extract metrics from logs
        local eosm3_tests=$(grep -o "Tests run: [0-9]*" /tmp/eosm3-test-results.log 2>/dev/null | awk '{print $3}' || echo "0")
        local eosmini_tests=$(grep -o "Tests run: [0-9]*" /tmp/eosmini-test-results.log 2>/dev/null | awk '{print $3}' || echo "0")
        local eosm3_time=$(grep "Duration:" /tmp/eosm3-test-results.log 2>/dev/null | awk '{print $2}' | sed 's/s//' || echo "0")
        local eosmini_time=$(grep "Duration:" /tmp/eosmini-test-results.log 2>/dev/null | awk '{print $2}' | sed 's/s//' || echo "0")

        # Create JSON output
        cat > "$output_file" <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "total_duration_seconds": $total_duration,
  "eosm3": {
    "test_count": $eosm3_tests,
    "duration_seconds": $eosm3_time
  },
  "eosmini": {
    "test_count": $eosmini_tests,
    "duration_seconds": $eosmini_time
  },
  "total_tests": $((eosm3_tests + eosmini_tests)),
  "sequential_estimate": $(echo "$eosm3_time + $eosmini_time" | bc),
  "speedup": $(echo "scale=2; ($eosm3_time + $eosmini_time) / $total_duration" | bc 2>/dev/null || echo "1.0")
}
EOF

        echo -e "${GREEN}✓ Metrics captured${NC}"
        return 0
    else
        echo -e "${RED}✗ Test run failed${NC}"
        return 1
    fi
}

# Function to compare metrics and detect regressions
check_regression() {
    local baseline_file="$1"
    local current_file="$2"

    if [ ! -f "$baseline_file" ]; then
        echo -e "${YELLOW}⚠ No baseline found. Creating baseline...${NC}"
        cp "$current_file" "$baseline_file"
        echo -e "${GREEN}✓ Baseline created at: $baseline_file${NC}"
        return 0
    fi

    echo ""
    echo -e "${CYAN}${BOLD}Performance Comparison${NC}"
    echo ""

    # Extract values using jq
    local baseline_duration=$(jq -r '.total_duration_seconds' "$baseline_file")
    local current_duration=$(jq -r '.total_duration_seconds' "$current_file")
    local baseline_speedup=$(jq -r '.speedup' "$baseline_file")
    local current_speedup=$(jq -r '.speedup' "$current_file")

    # Calculate percentage changes
    local duration_change=$(echo "scale=1; (($current_duration - $baseline_duration) / $baseline_duration) * 100" | bc)
    local speedup_change=$(echo "scale=1; (($current_speedup - $baseline_speedup) / $baseline_speedup) * 100" | bc)

    # Display comparison
    echo -e "  ${BOLD}Total Duration:${NC}"
    echo -e "    Baseline: ${baseline_duration}s"
    echo -e "    Current:  ${current_duration}s"
    echo -e "    Change:   ${duration_change}%"
    echo ""

    echo -e "  ${BOLD}Speedup Factor:${NC}"
    echo -e "    Baseline: ${baseline_speedup}x"
    echo -e "    Current:  ${current_speedup}x"
    echo -e "    Change:   ${speedup_change}%"
    echo ""

    # Check for regression
    local has_regression=false

    # Duration regression (slower)
    if (( $(echo "$duration_change > $REGRESSION_THRESHOLD" | bc -l) )); then
        echo -e "${RED}${BOLD}❌ REGRESSION DETECTED: Duration increased by ${duration_change}%${NC}"
        has_regression=true
    fi

    # Speedup regression (less efficient)
    if (( $(echo "$speedup_change < -$REGRESSION_THRESHOLD" | bc -l) )); then
        echo -e "${RED}${BOLD}❌ REGRESSION DETECTED: Speedup decreased by ${speedup_change}%${NC}"
        has_regression=true
    fi

    if [ "$has_regression" = true ]; then
        echo ""
        echo -e "${YELLOW}Recommended Actions:${NC}"
        echo -e "  1. Review recent code changes"
        echo -e "  2. Check system resource usage"
        echo -e "  3. Verify SSH connectivity to EOSmini"
        echo -e "  4. Run: ~/bin/check-system-sync.sh"
        echo ""
        return 1
    else
        echo -e "${GREEN}${BOLD}✅ No performance regression detected${NC}"
        echo ""
        return 0
    fi
}

# Main execution
main() {
    # Capture current metrics (skip if --compare-only)
    if [ "$COMPARE_ONLY" = false ]; then
        if ! capture_metrics "$CURRENT_RUN_FILE"; then
            exit 1
        fi
    else
        echo -e "${YELLOW}Skipping metric capture (--compare-only mode)${NC}"
        if [ ! -f "$CURRENT_RUN_FILE" ]; then
            echo -e "${RED}ERROR: Current metrics file not found: $CURRENT_RUN_FILE${NC}"
            exit 1
        fi
    fi

    # Compare against baseline
    if check_regression "$BASELINE_FILE" "$CURRENT_RUN_FILE"; then
        echo -e "${GREEN}Performance test PASSED${NC}"
        exit 0
    else
        echo -e "${RED}Performance test FAILED${NC}"
        exit 1
    fi
}

# Run main function
main
