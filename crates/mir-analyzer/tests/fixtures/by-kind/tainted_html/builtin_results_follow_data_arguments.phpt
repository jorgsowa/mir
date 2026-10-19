===description===
A builtin's result carries the taint of the arguments its value is built from,
not of its offsets, counts, formats or placeholders. `base64_encode` leaves
nothing HTML-, header- or cookie-injectable.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <PossiblyFalseOperand errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function dataArgumentStillCarries(): void {
    echo substr($_GET['a'], 0, 3);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_repeat($_GET['a'], 2);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_pad('x', 10, $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_pad($_GET['a'], 10);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo date($_GET['f']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo substr_replace('abc', $_GET['a'], 1);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo implode(',', array_fill(0, 2, $_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo implode(',', explode(',', $_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

/** @throws \Exception */
function countsAndFormatsDoNotCarry(): void {
    echo substr('abc', 0, $_GET['n']);
    echo str_repeat('ab', $_GET['n']);
    echo str_pad('ab', $_GET['n']);
    echo date('Y-m-d', $_GET['ts']);
    echo implode(',', array_fill(0, $_GET['n'], '?'));
    echo implode(',', range(1, $_GET['n']));
    echo get_debug_type($_GET['a']);
    echo gettype($_GET['a']);
    echo explode($_GET['sep'], 'a,b')[0];
    echo trim('  x  ', $_GET['chars']);
    echo bin2hex(random_bytes($_GET['n']));
}

function encodedOutput(): void {
    echo base64_encode($_GET['a']);
    echo base64_encode((string) json_encode($_GET));
    header('X-Data: ' . base64_encode($_GET['a']));
    setcookie('c', base64_encode($_GET['a']));
    echo bin2hex($_GET['a']);
}

function base64StillCarriesOtherKinds(mysqli $db): void {
    mysqli_query($db, 'SELECT ' . base64_encode($_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}

function inPlaceholderPattern(mysqli $db): void {
    $in = implode(',', array_fill(0, count($_GET['ids']), '?'));
    mysqli_query($db, "SELECT * FROM t WHERE id IN ($in)");
}

function throughUserFunctions(): void {
    echo firstChars($_GET['a'], 3);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo firstChars('abc', $_GET['n']);
    echo encode($_GET['a']);
}

function firstChars(string $s, int $n): string {
    return substr($s, 0, $n);
}

function encode(string $s): string {
    return base64_encode($s);
}

function namedAndSpreadArgumentsStayConservative(): void {
    echo substr(string: 'abc', offset: 0, length: $_GET['n']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo substr(...[$_GET['a'], 0]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
