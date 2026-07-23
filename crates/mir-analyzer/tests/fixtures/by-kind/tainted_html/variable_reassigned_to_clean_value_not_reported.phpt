===description===
Taint on a plain variable was additive-only — once tainted, a variable
stayed tainted forever even after being reassigned to a proven-clean value
(a literal, or the result of a sanitizing cast). Mirrors the property-taint
arm, which already clears stale taint on a clean overwrite. A reassignment
to another tainted source must still leave the variable tainted.
===config===
suppress=MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function reassignedToLiteralIsClean(): void {
    $x = $_GET['x'];
    $x = "safe";
    echo $x;
}

function reassignedToIntCastIsClean(): void {
    $x = $_GET['x'];
    $x = (int) $x;
    echo $x;
}

function reassignedToTaintedStaysTainted(): void {
    $x = $_GET['x'];
    $x = $_POST['y'];
    echo $x;
}
===expect===
TaintedHtml@17:4-17:12: Tainted HTML output — possible XSS
