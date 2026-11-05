===description===
A passing anchored `preg_match` bounds the captures as it bounds the subject; a failed one, or an unanchored pattern, doesn't.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function captured(): void {
    if (preg_match('/^(\d+)$/', $_GET['x'], $m)) {
        echo $m[1];
    }
}

function comparedResult(): void {
    $x = $_GET['x'];
    if (preg_match('/^(\d+)-(\d+)$/', $x, $m) === 1) {
        echo $m[1], $m[2];
    }
}

function failed(): void {
    if (!preg_match('/^(\d+)$/', $_GET['x'], $m)) {
        echo $_GET['x'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function unanchored(): void {
    if (preg_match('/(.+)/', $_GET['x'], $m)) {
        echo $m[1];
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
