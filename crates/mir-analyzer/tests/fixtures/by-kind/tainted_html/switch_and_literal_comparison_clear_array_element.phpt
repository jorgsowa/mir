===description===
Inside a `case 'x':` body, or where `$_GET['a'] === 'x'` or
`in_array($_GET['a'], [..], true)` holds, the element is the text the program
spells out. A `default:` body, a body entered by falling through from another
body, non-literal labels, numeric labels (for the kinds whitespace breaks) and
the loose `==` / `in_array()` forms stay tainted.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyUndefinedArrayOffset errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const KIND = 'k';

function switchOnElement(): void {
    switch ($_GET['a']) {
        case 'x':
            echo $_GET['a'];
            break;
        default:
            echo $_GET['a'];
    }
}

function severalLabelsOneBody(): void {
    switch ($_GET['a']) {
        case 'x':
        case 'y':
            echo $_GET['a'];
            break;
    }
}

function severalLabelsOneNonLiteral(string $other): void {
    switch ($_GET['a']) {
        case 'x':
        case $other:
            echo $_GET['a'];
            break;
    }
}

function fallingThroughKeepsTaint(): void {
    switch ($_GET['a']) {
        case 'x':
            $y = 1;
        case 'y':
            echo $_GET['a'];
            break;
    }
}

function defaultFallingIntoCase(): void {
    switch ($_GET['a']) {
        default:
            $y = 1;
        case 'x':
            echo $_GET['a'];
    }
}

function returnEndsTheBody(): string {
    switch ($_GET['a']) {
        case 'x':
            return 'one';
        case 'y':
            echo $_GET['a'];
            return 'two';
    }
    return '';
}

function constantLabel(): void {
    switch ($_GET['a']) {
        case KIND:
            echo $_GET['a'];
    }
}

function numericLabels(): void {
    switch ($_GET['a']) {
        case 1:
            echo $_GET['a'];
            header('X-Page: ' . $_GET['a']);
            break;
        case '2':
            echo $_GET['a'];
            break;
    }
}

function otherElementUnaffected(): void {
    switch ($_GET['a']) {
        case 'x':
            echo $_GET['b'];
    }
}

function switchTrueUnaffected(): void {
    switch (true) {
        case $_GET['a'] === 'x':
            echo $_GET['a'];
            break;
        case ctype_digit($_GET['b']):
            echo $_GET['b'];
            break;
        case $_GET['c'] !== 'x':
            echo $_GET['c'];
            break;
    }
}

function switchOnLocalArrayElement(): void {
    $data = ['mode' => $_GET['mode'], 'name' => $_GET['name']];
    switch ($data['mode']) {
        case 'a':
            echo $data['mode'];
            echo $data['name'];
    }
}

function switchOnVariable(): void {
    $a = $_GET['a'];
    switch ($a) {
        case 'x':
            echo $a;
            break;
        default:
            echo $a;
    }
}

function strictComparison(): void {
    if ($_GET['a'] === 'x') {
        echo $_GET['a'];
    }
    if ('y' === $_GET['b']) {
        echo $_GET['b'];
    }
    if ($_GET['c'] !== 'x') {
        return;
    }
    echo $_GET['c'];
}

function strictComparisonOtherBranch(): void {
    if ($_GET['a'] === 'x') {
        echo 'fine';
    } else {
        echo $_GET['a'];
    }
    if ($_GET['b'] !== 'x') {
        echo $_GET['b'];
    }
}

function looseComparison(): void {
    if ($_GET['a'] == 'x') {
        echo $_GET['a'];
    }
    if ($_GET['n'] == 10) {
        echo $_GET['n'];
        header('X-N: ' . $_GET['n']);
    }
}

function comparisonWithNonLiteral(string $expected): void {
    if ($_GET['a'] === $expected) {
        echo $_GET['a'];
    }
}

function comparisonOr(): void {
    if ($_GET['a'] === 'x' || $_GET['a'] === 'y') {
        echo $_GET['a'];
    }
}

function comparisonAnd(): void {
    if ($_GET['a'] === 'x' && $_GET['b'] === 'y') {
        echo $_GET['a'];
        echo $_GET['b'];
        echo $_GET['c'];
    }
}

function inArrayStrict(): void {
    if (in_array($_GET['a'], ['x', 'y'], true)) {
        echo $_GET['a'];
    }
    if (!in_array($_GET['b'], ['x', 'y'], true)) {
        return;
    }
    echo $_GET['b'];
}

function inArrayLoose(): void {
    if (in_array($_GET['a'], ['x', 'y'])) {
        echo $_GET['a'];
    }
    if (in_array($_GET['n'], [1, 2, 3])) {
        echo $_GET['n'];
        header('X-N: ' . $_GET['n']);
    }
}

function inArrayWithNonLiteralList(array $allowed, string $dynamic): void {
    if (in_array($_GET['a'], $allowed, true)) {
        echo $_GET['a'];
    }
    if (in_array($_GET['b'], ['x', $dynamic], true)) {
        echo $_GET['b'];
    }
}

function inArrayElseBranch(): void {
    if (in_array($_GET['a'], ['x', 'y'], true)) {
        echo 'ok';
    } else {
        echo $_GET['a'];
    }
}

function ternaries(): void {
    echo $_GET['a'] === 'x' ? $_GET['a'] : '';
    echo in_array($_GET['b'], ['x'], true) ? $_GET['b'] : '';
    echo $_GET['c'] === 'x' ? '' : $_GET['c'];
}
===expect===
TaintedHtml@10:12-10:28: Tainted HTML output — possible XSS
TaintedHtml@27:12-27:28: Tainted HTML output — possible XSS
TaintedHtml@37:12-37:28: Tainted HTML output — possible XSS
TaintedHtml@47:12-47:28: Tainted HTML output — possible XSS
TaintedHtml@65:12-65:28: Tainted HTML output — possible XSS
TaintedHeader@73:12-73:43: Tainted HTTP header — possible header injection or open redirect
TaintedHtml@84:12-84:28: Tainted HTML output — possible XSS
TaintedHtml@97:12-97:28: Tainted HTML output — possible XSS
TaintedHtml@107:12-107:31: Tainted HTML output — possible XSS
TaintedHtml@118:12-118:20: Tainted HTML output — possible XSS
TaintedHtml@139:8-139:24: Tainted HTML output — possible XSS
TaintedHtml@142:8-142:24: Tainted HTML output — possible XSS
TaintedHtml@148:8-148:24: Tainted HTML output — possible XSS
TaintedHtml@151:8-151:24: Tainted HTML output — possible XSS
TaintedHeader@152:8-152:36: Tainted HTTP header — possible header injection or open redirect
TaintedHtml@158:8-158:24: Tainted HTML output — possible XSS
TaintedHtml@172:8-172:24: Tainted HTML output — possible XSS
TaintedHtml@188:8-188:24: Tainted HTML output — possible XSS
TaintedHtml@191:8-191:24: Tainted HTML output — possible XSS
TaintedHeader@192:8-192:36: Tainted HTTP header — possible header injection or open redirect
TaintedHtml@198:8-198:24: Tainted HTML output — possible XSS
TaintedHtml@201:8-201:24: Tainted HTML output — possible XSS
TaintedHtml@209:8-209:24: Tainted HTML output — possible XSS
TaintedHtml@216:4-216:46: Tainted HTML output — possible XSS
