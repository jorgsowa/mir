===description===
`trim($x) === 'a'` lets a line break around the value through; a loose number comparison lets
padding through. A strict match clears the value.
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
function trimmed(): void {
    $x = $_GET['x'];
    if (trim($x) === 'a') {
        header('X-A: ' . $x);
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }
}

function looseNumber(): void {
    $x = $_GET['x'];
    if ($x == 1) {
        header('X-A: ' . $x);
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }
}

function strict(): void {
    $x = $_GET['x'];
    if ($x === 'a') {
        header('X-A: ' . $x);
    }
}
