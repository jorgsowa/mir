===description===
A stored preg_match only vouches for its captures until something writes to them.
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
function overwritten(): void {
    $x = $_GET['a'];
    $ok = preg_match('/^[a-z]+$/D', $x, $m);
    $m = $_GET['q'];
    if ($ok) {
        echo $m;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function element_overwritten(): void {
    $x = $_GET['a'];
    $ok = preg_match('/^[a-z]+$/D', $x, $m);
    $m[0] = $_GET['q'];
    if ($ok) {
        echo $m[0];
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function rewritten_by_another_match(): void {
    $x = $_GET['a'];
    $y = $_GET['b'];
    $ok = preg_match('/^[a-z]+$/D', $x, $m);
    preg_match('/(.*)/', $y, $m);
    if ($ok) {
        echo $m[1];
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function untouched_captures_stay_clean(): void {
    $x = $_GET['a'];
    $ok = preg_match('/^[a-z]+$/D', $x, $m);
    if ($ok) {
        echo $m[0];
    }
}
