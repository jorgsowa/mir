===description===
Clamping an amount to a maximum, also inside an either-bound check, caps it.
===config===
<mir>
  <issueHandlers>
    <ArgumentTypeCoercion errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function clamped(): string {
    $n = (int) $_GET['n'];
    if ($n > 100) {
        $n = 100;
    }
    return str_repeat('x', $n);
}

function clampedBothWays(): string {
    $n = (int) $_GET['n'];
    if ($n < 1 || $n > 100) {
        $n = 10;
    }
    return str_repeat('x', $n);
}

function reassignedInBothBranches(): string {
    $n = (int) $_GET['n'];
    if ($n > 100) {
        $n = 100;
    } else {
        $n = 5;
    }
    return str_repeat('x', $n);
}
