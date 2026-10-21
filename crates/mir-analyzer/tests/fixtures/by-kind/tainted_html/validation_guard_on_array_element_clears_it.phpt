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
    }
}

function otherKeyStaysTainted(): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    echo $_GET['b'];
}

function otherSuperglobalStaysTainted(): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    echo $_POST['a'];
}

function unguardedPathMerges(bool $c): void {
    if ($c) {
        if (!ctype_digit($_GET['a'])) {
            return;
        }
    }
    echo $_GET['a'];
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
}

function wholeArrayReassigned(array $input): void {
    if (!ctype_digit($_GET['a'])) {
        return;
    }
    $_GET = $input;
    echo $_GET['a'];
}

function guardOnAnotherKeyDoesNotClear(): void {
    if (ctype_digit($_GET['a'])) {
        echo $_GET['b'];
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
}

function unanchoredRegexIsNoGuard(): void {
    if (preg_match('/[a-z]+/', $_GET['name']) !== 1) {
        return;
    }
    echo $_GET['name'];
}

function ternaryBranch(): void {
    echo ctype_digit($_GET['a']) ? $_GET['a'] : '';
    echo !ctype_digit($_GET['a']) ? '' : $_GET['a'];
    echo ctype_digit($_GET['a']) ? '' : $_GET['a'];
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
    }
    if (preg_match('/^\d+$/D', $_GET['b']) === 0) {
        echo $_GET['b'];
    }
    if (preg_match('/^\d+$/D', $_GET['c']) !== 1) {
        echo $_GET['c'];
    }
    if (preg_match('/^\d+$/D', $_GET['e']) >= 0) {
        echo $_GET['e'];
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
    }
}

function plainVariableComparison(): void {
    $v = $_GET['v'];
    if (preg_match('/^\d+$/D', $v) !== 1) {
        return;
    }
    echo $v;
}
===expect===
TaintedHtml@13:8-13:26: Tainted HTML output — possible XSS
TaintedHtml@21:4-21:20: Tainted HTML output — possible XSS
TaintedHtml@28:4-28:21: Tainted HTML output — possible XSS
TaintedHtml@37:4-37:20: Tainted HTML output — possible XSS
TaintedHtml@58:4-58:20: Tainted HTML output — possible XSS
TaintedHtml@66:4-66:20: Tainted HTML output — possible XSS
TaintedHtml@71:8-71:24: Tainted HTML output — possible XSS
TaintedHeader@99:4-99:38: Tainted HTTP header — possible header injection or open redirect
TaintedHtml@106:4-106:23: Tainted HTML output — possible XSS
TaintedHtml@112:4-112:51: Tainted HTML output — possible XSS
TaintedHtml@130:4-130:23: Tainted HTML output — possible XSS
TaintedHtml@179:8-179:24: Tainted HTML output — possible XSS
TaintedHtml@182:8-182:24: Tainted HTML output — possible XSS
TaintedHtml@185:8-185:24: Tainted HTML output — possible XSS
TaintedHtml@188:8-188:24: Tainted HTML output — possible XSS
TaintedHtml@204:8-204:24: Tainted HTML output — possible XSS
