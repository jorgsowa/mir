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
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
            break;
    }
}

function fallingThroughKeepsTaint(): void {
    switch ($_GET['a']) {
        case 'x':
            $y = 1;
        case 'y':
            echo $_GET['a'];
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
            break;
    }
}

function defaultFallingIntoCase(): void {
    switch ($_GET['a']) {
        default:
            $y = 1;
        case 'x':
            echo $_GET['a'];
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function numericLabels(): void {
    switch ($_GET['a']) {
        case 1:
            echo $_GET['a'];
            header('X-Page: ' . $_GET['a']);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
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
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
            break;
    }
}

function switchOnLocalArrayElement(): void {
    $data = ['mode' => $_GET['mode'], 'name' => $_GET['name']];
    switch ($data['mode']) {
        case 'a':
            echo $data['mode'];
            echo $data['name'];
//          ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//          ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if ($_GET['b'] !== 'x') {
        echo $_GET['b'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function looseComparison(): void {
    if ($_GET['a'] == 'x') {
        echo $_GET['a'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if ($_GET['n'] == 10) {
        echo $_GET['n'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        header('X-N: ' . $_GET['n']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }
}

function comparisonWithNonLiteral(string $expected): void {
    if ($_GET['a'] === $expected) {
        echo $_GET['a'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if (in_array($_GET['n'], [1, 2, 3])) {
        echo $_GET['n'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        header('X-N: ' . $_GET['n']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }
}

function inArrayWithNonLiteralList(array $allowed, string $dynamic): void {
    if (in_array($_GET['a'], $allowed, true)) {
        echo $_GET['a'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if (in_array($_GET['b'], ['x', $dynamic], true)) {
        echo $_GET['b'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function inArrayElseBranch(): void {
    if (in_array($_GET['a'], ['x', 'y'], true)) {
        echo 'ok';
    } else {
        echo $_GET['a'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function ternaries(): void {
    echo $_GET['a'] === 'x' ? $_GET['a'] : '';
    echo in_array($_GET['b'], ['x'], true) ? $_GET['b'] : '';
    echo $_GET['c'] === 'x' ? '' : $_GET['c'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
