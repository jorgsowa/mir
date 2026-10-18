===description===
A user function's result carries the taint of what it returns: a source it
reads itself, or the arguments that flow into its return value. Sanitizing
inside the function, or a return type that can't carry a payload, ends it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedInferredReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function readQuery(): string { return (string) $_GET['q']; }
function identity(string $v): string { return $v; }
function wrap(string $v): string { return '<b>' . identity($v) . '</b>'; }
function second(string $a, string $b): string { return $b; }
function escaped(string $v): string { return htmlspecialchars($v); }
function length(string $v): int { return strlen($v); }
function constant(string $v): string { return 'fixed'; }
function untyped($v) { return trim($v); }

function sources(): void {
    echo readQuery();
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $q = identity($_GET['q']);
    /** @mir-check $q is string */
    echo $q;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo wrap($_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo untyped($_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo second($_GET['q'], 'safe');
    echo second('safe', $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo second(b: $_GET['q'], a: 'safe');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function sanitized(mysqli $db): void {
    echo escaped($_GET['q']);
    mysqli_query($db, 'SELECT ' . escaped($_GET['q']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    $len = length($_GET['q']);
    /** @mir-check $len is int */
    echo $len;
    echo constant($_GET['q']);
    echo identity('literal');
}

function countdown(string $v, int $n): string {
    return $n > 0 ? countdown($v, $n - 1) : $v;
}

function recursive(): void {
    echo countdown($_GET['q'], 3);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function gen(string $v): Generator {
    yield 1;
    return $v;
}

function generatorReturnIsNotTheCallResult(): void {
    foreach (gen($_GET['q']) as $item) {
        echo $item;
    }
}
