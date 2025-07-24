#!/bin/bash

echo "🔒 Running Security Tests..."
echo "=========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Check for dangerouslySetInnerHTML usage
echo -e "\n${YELLOW}Test 1: Checking for unsanitized dangerouslySetInnerHTML usage...${NC}"
DANGEROUS_COUNT=$(grep -r "dangerouslySetInnerHTML" src/ --include="*.tsx" --include="*.jsx" | grep -v "sanitize" | wc -l)
if [ $DANGEROUS_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ No unsanitized dangerouslySetInnerHTML found${NC}"
else
    echo -e "${RED}✗ Found $DANGEROUS_COUNT instances of potentially unsanitized dangerouslySetInnerHTML${NC}"
    grep -r "dangerouslySetInnerHTML" src/ --include="*.tsx" --include="*.jsx" | grep -v "sanitize"
fi

# Test 2: Check for DOMPurify import
echo -e "\n${YELLOW}Test 2: Checking DOMPurify usage...${NC}"
DOMPURIFY_IMPORTS=$(grep -r "import.*DOMPurify" src/ --include="*.ts" --include="*.tsx" | wc -l)
if [ $DOMPURIFY_IMPORTS -gt 0 ]; then
    echo -e "${GREEN}✓ DOMPurify is imported in $DOMPURIFY_IMPORTS files${NC}"
else
    echo -e "${RED}✗ DOMPurify not found in project${NC}"
fi

# Test 3: Check for eval usage
echo -e "\n${YELLOW}Test 3: Checking for dangerous eval() usage...${NC}"
EVAL_COUNT=$(grep -r "eval(" src/ --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx" | wc -l)
if [ $EVAL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ No eval() usage found${NC}"
else
    echo -e "${RED}✗ Found $EVAL_COUNT instances of eval()${NC}"
    grep -r "eval(" src/ --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx"
fi

# Test 4: Check for innerHTML usage
echo -e "\n${YELLOW}Test 4: Checking for innerHTML usage...${NC}"
INNERHTML_COUNT=$(grep -r "innerHTML" src/ --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx" | wc -l)
if [ $INNERHTML_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ No innerHTML usage found${NC}"
else
    echo -e "${RED}✗ Found $INNERHTML_COUNT instances of innerHTML${NC}"
    grep -r "innerHTML" src/ --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx"
fi

# Test 5: Check CSP middleware
echo -e "\n${YELLOW}Test 5: Checking Content Security Policy...${NC}"
if [ -f "src/middleware.ts" ]; then
    CSP_HEADER=$(grep -c "Content-Security-Policy" src/middleware.ts)
    if [ $CSP_HEADER -gt 0 ]; then
        echo -e "${GREEN}✓ CSP headers configured in middleware${NC}"
    else
        echo -e "${RED}✗ CSP headers not found in middleware${NC}"
    fi
else
    echo -e "${RED}✗ Middleware file not found${NC}"
fi

# Test 6: Check for javascript: protocol
echo -e "\n${YELLOW}Test 6: Checking for javascript: protocol usage...${NC}"
JS_PROTOCOL=$(grep -r "javascript:" src/ --include="*.ts" --include="*.tsx" | grep -v "test" | grep -v "comment" | wc -l)
if [ $JS_PROTOCOL -eq 0 ]; then
    echo -e "${GREEN}✓ No javascript: protocol usage found${NC}"
else
    echo -e "${YELLOW}⚠ Found $JS_PROTOCOL instances of javascript: protocol (verify if properly handled)${NC}"
    grep -r "javascript:" src/ --include="*.ts" --include="*.tsx" | grep -v "test" | grep -v "comment"
fi

# Test 7: Run npm audit
echo -e "\n${YELLOW}Test 7: Running npm audit for known vulnerabilities...${NC}"
cd /root/repo/legacybridge
npm audit --production 2>/dev/null
AUDIT_EXIT=$?
if [ $AUDIT_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ No vulnerabilities found in dependencies${NC}"
else
    echo -e "${YELLOW}⚠ Some vulnerabilities found in dependencies (see above)${NC}"
fi

# Test 8: Check for sensitive data patterns
echo -e "\n${YELLOW}Test 8: Checking for hardcoded sensitive data...${NC}"
SENSITIVE_PATTERNS="password|secret|api_key|apikey|token|private_key"
SENSITIVE_COUNT=$(grep -rEi "$SENSITIVE_PATTERNS" src/ --include="*.ts" --include="*.tsx" | grep -v "test" | grep -v "interface" | grep -v "type" | wc -l)
if [ $SENSITIVE_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ No hardcoded sensitive data patterns found${NC}"
else
    echo -e "${YELLOW}⚠ Found $SENSITIVE_COUNT instances of potential sensitive data patterns${NC}"
fi

# Summary
echo -e "\n${YELLOW}=========================="
echo "Security Test Summary"
echo -e "==========================${NC}"

ISSUES=0
[ $DANGEROUS_COUNT -gt 0 ] && ISSUES=$((ISSUES + 1))
[ $DOMPURIFY_IMPORTS -eq 0 ] && ISSUES=$((ISSUES + 1))
[ $EVAL_COUNT -gt 0 ] && ISSUES=$((ISSUES + 1))
[ $INNERHTML_COUNT -gt 0 ] && ISSUES=$((ISSUES + 1))
[ ! -f "src/middleware.ts" ] && ISSUES=$((ISSUES + 1))
[ $AUDIT_EXIT -ne 0 ] && ISSUES=$((ISSUES + 1))

if [ $ISSUES -eq 0 ]; then
    echo -e "${GREEN}✓ All security tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $ISSUES security issues that need attention${NC}"
    exit 1
fi