===description===
A validation guard (`ctype_digit`, an anchored `preg_match`) on an array element
-- a superglobal entry or a variable's literal key -- clears that element for the
kinds the guard proves, on the path where it passed. Other keys, the other
branch of a merge, and a later overwrite are unaffected.
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
  </issueHandlers>
</mir>
===file===
<?php
function earlyReturn(): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    echo $_GET['a'];
}

function thenBranch(): void {
    if (ctype_digit($_POST['id'])) {
        echo $_POST['id'];
    } else {
        echo $_POST['id'];
//      ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function otherKeyStaysTainted(): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    echo $_GET['b'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function otherSuperglobalStaysTainted(): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    echo $_POST['a'];
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function unguardedPathMerges(bool $c): void {
    if ($c) {
        if (!ctype_digit($_GET['a'])) {
            return;
        }
    }
    echo $_GET['a'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function guardedOnBothPaths(bool $c): void {
    if ($c) {
        if (!ctype_digit($_GET['a'])) {
            return;
        }
    } else {
        if (!ctype_digit($_GET['a'])) {
            return;
        }
    }
    echo $_GET['a'];
}

function overwrittenAfterGuard(): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    $_GET['a'] = $_GET['b'];
    echo $_GET['a'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function wholeArrayReassigned(array $input): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    $_GET = $input;
    echo $_GET['a'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function guardOnAnotherKeyDoesNotClear(): void {
    if (ctype_digit($_GET['a'])) {
        echo $_GET['b'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function nullCoalesceSubject(): void {
    if (!ctype_digit($_GET['n'] ?? '')) {
        return;
    }
    echo $_GET['n'];
}

function andChain(): void {
    if (isset($_GET['n']) && ctype_digit($_GET['n'])) {
        echo $_GET['n'];
    }
}

function regexGuard(): void {
    if (preg_match('/^[a-z0-9_]+$/D', $_GET['name']) !== 1) {
        return;
    }
    echo $_GET['name'];
}

function regexGuardDoesNotProveHeaderSafety(): void {
    if (preg_match('/^[a-z]+$/', $_GET['name']) !== 1) {
        return;
    }
    header('X-Name: ' . $_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}

function unanchoredRegexIsNoGuard(): void {
    if (preg_match('/[a-z]+/', $_GET['name']) !== 1) {
        return;
    }
    echo $_GET['name'];
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function ternaryBranch(): void {
    echo ctype_digit($_GET['a']) ? $_GET['a'] : '';
    echo !ctype_digit($_GET['a']) ? '' : $_GET['a'];
    echo ctype_digit($_GET['a']) ? '' : $_GET['a'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function loopBody(): void {
    foreach (['a', 'b'] as $_) {
        if (!ctype_digit($_GET['a'])) {
            continue;
        }
        echo $_GET['a'];
    }
}

function localArrayElement(): void {
    $data = ['id' => $_GET['id'], 'name' => $_GET['name']];
    if (!ctype_digit($data['id'])) {
        return;
    }
    echo $data['id'];
    echo $data['name'];
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function paramArrayElement(array $row): void {
    if (!ctype_digit($row['id'])) {
        return;
    }
    echo $row['id'];
}

function guardedElementPassedToCallee(): void {
    if (!ctype_digit($_GET['id'])) {
        return;
    }
    show($_GET['id']);
}

function show(string $s): void {
    echo $s;
}

function comparisonsWithMatchResult(): void {
    if (preg_match('/^\d+$/D', $_GET['a']) === 1) {
        echo $_GET['a'];
    }
    if (preg_match('/^\d+$/D', $_GET['b']) > 0) {
        echo $_GET['b'];
    }
    if (1 === preg_match('/^\d+$/D', $_GET['c'])) {
        echo $_GET['c'];
    }
    if (preg_match('/^\d+$/D', $_GET['d']) != 0) {
        echo $_GET['d'];
    }
    if (preg_match('/^\d+$/D', $_GET['e']) == 0) {
        return;
    }
    echo $_GET['e'];
    if (preg_match('/^\d+$/D', $_GET['f']) < 1) {
        return;
    }
    echo $_GET['f'];
    if (0 < preg_match('/^\d+$/D', $_GET['g'])) {
        echo $_GET['g'];
    }
}

function comparisonsThatProveNothing(): void {
    if (preg_match('/^\d+$/D', $_GET['a']) !== 0) {
        echo $_GET['a'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if (preg_match('/^\d+$/D', $_GET['b']) === 0) {
        echo $_GET['b'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if (preg_match('/^\d+$/D', $_GET['c']) !== 1) {
        echo $_GET['c'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    if (preg_match('/^\d+$/D', $_GET['e']) >= 0) {
        echo $_GET['e'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function ctypeComparedWithBool(): void {
    if (ctype_digit($_GET['a']) === true) {
        echo $_GET['a'];
    }
    if (ctype_digit($_GET['b']) !== false) {
        echo $_GET['b'];
    }
    if (ctype_digit($_GET['c']) === false) {
        return;
    }
    echo $_GET['c'];
    if (ctype_digit($_GET['d']) === false) {
        echo $_GET['d'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function plainVariableComparison(): void {
    $v = $_GET['v'];
    if (preg_match('/^\d+$/D', $v) !== 1) {
        return;
    }
    echo $v;
}
