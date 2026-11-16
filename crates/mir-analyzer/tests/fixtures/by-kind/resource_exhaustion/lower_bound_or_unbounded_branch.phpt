===description===
A lower bound or a non-zero check leaves the amount uncapped, and so does the
branch where an upper-bound check fails.
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
function positive(): string {
    $n = (int) $_GET['n'];
    if ($n > 0) {
        return str_repeat('x', $n);
//             ^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: str_repeat() takes its size, count or duration straight from the request — possible denial of service
    }
    return '';
}

function nonZero(): string {
    $n = (int) $_GET['n'];
    if ($n !== 0) {
        return str_repeat('x', $n);
//             ^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: str_repeat() takes its size, count or duration straight from the request — possible denial of service
    }
    return '';
}

function overTheCap(): array {
    $n = (int) $_GET['n'];
    if ($n <= 100) {
        return [];
    } else {
        return array_fill(0, $n, 0);
//             ^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: array_fill() takes its size, count or duration straight from the request — possible denial of service
    }
}
