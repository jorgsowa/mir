===description===
`$$name` where `$name` holds a known literal string always returned bare
`mixed` and was never checked for taint at all — both the real type and
the taint state of the referenced variable are resolvable when the name
is a literal string, so this narrow case is no longer treated as opaque.
===config===
suppress=MixedArgument,UnusedVariable,MixedAssignment,MixedArrayAccess
===file===
<?php
function typeIsResolved(): void {
    $x = 5;
    $name = 'x';
    $val = $$name;
    /** @mir-check $val is int */
    $_ = 1;
}

function taintPropagates(): void {
    $x = $_GET['input'];
    $name = 'x';
    echo $$name;
}
===expect===
TaintedHtml@13:4-13:16: Tainted HTML output — possible XSS
