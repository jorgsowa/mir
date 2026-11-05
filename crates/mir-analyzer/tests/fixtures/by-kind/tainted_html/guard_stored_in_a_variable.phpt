===description===
A validation guard stored in a variable clears its subject when the variable is tested, unless
the subject changed in between.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function stored(): void {
    $x = $_GET['x'];
    $ok = ctype_digit($x);
    if ($ok) {
        echo $x;
    }
}

function comparedResult(): void {
    $x = $_GET['x'];
    $ok = preg_match('/^\d+$/', $x) === 1;
    if (!$ok) {
        exit;
    }
    echo $x;
}

function element(): void {
    $ok = ctype_digit($_GET['id']);
    if ($ok) {
        echo $_GET['id'];
    }
}

function reassigned(): void {
    $x = $_GET['x'];
    $ok = ctype_digit($x);
    $x = $_GET['y'];
    if ($ok) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function elementRewritten(): void {
    $ok = ctype_digit($_GET['id']);
    $_GET['id'] = $_POST['id'];
    if ($ok) {
        echo $_GET['id'];
//      ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function failingPath(): void {
    $x = $_GET['x'];
    $ok = ctype_digit($x);
    if (!$ok) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
