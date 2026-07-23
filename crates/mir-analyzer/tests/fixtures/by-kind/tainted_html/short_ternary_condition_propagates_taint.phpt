===description===
`$x ?: $y` (short ternary) has its true-branch VALUE be the condition
itself -- `then_expr` is None for this form -- but is_expr_tainted's
Ternary arm's `is_some_and` on a None then_expr was unconditionally
false, so the condition's own taint was never checked at all.
===config===
suppress=MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    $name = $_GET['name'] ?: 'anon';
    echo $name;
}
===expect===
TaintedHtml@4:4-4:15: Tainted HTML output — possible XSS
