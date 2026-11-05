===description===
A guard sees through `(string)`, a case change or `trim` on its subject, and accepts a loose
comparison with a non-numeric literal. A numeric literal keeps padded-whitespace kinds tainted.
===config===
<mir>
  <issueHandlers>
    <ArgumentTypeCoercion errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function cast(): void {
    $x = $_GET['x'];
    if (ctype_digit((string) $x)) {
        echo $x;
    }
}

function lowered(): void {
    $x = $_GET['x'];
    if (strtolower($x) === 'jpg') {
        echo $x;
    }
}

function trimmed(): void {
    $x = $_GET['x'];
    if (trim($x) === 'a') {
        echo $x;
    }
}

function loose(): void {
    $x = $_GET['x'];
    if ($x == 'a') {
        echo $x;
    }
}

function looseList(): void {
    $x = $_GET['x'];
    if (in_array($x, ['a', 'b'])) {
        echo $x;
    }
}

function loweredList(): void {
    $x = $_GET['x'];
    if (in_array(strtolower($x), ['jpg', 'png'], true)) {
        echo $x;
    }
}

function intCast(): void {
    $x = $_GET['x'];
    if (ctype_digit((int) $x)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function trimCharacters(): void {
    $x = $_GET['x'];
    if (trim($x, '<>') === 'a') {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function otherFunction(): void {
    $x = $_GET['x'];
    if (strrev($x) === 'a') {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function looseFailingPath(): void {
    $x = $_GET['x'];
    if ($x != 'a') {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
