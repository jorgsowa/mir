===description===
Reassigning the subject of a stored check inside a condition invalidates the check after the branches merge.
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
function assigned_in_condition(): void {
    $x = $_GET['a'];
    $ok = ctype_digit($x);
    if ($x = $_GET['q']) {
    }
    if ($ok) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function untouched_subject_keeps_the_check(bool $c): void {
    $x = $_GET['a'];
    $ok = ctype_digit($x);
    if ($c) {
    }
    if ($ok) {
        echo $x;
    }
}
