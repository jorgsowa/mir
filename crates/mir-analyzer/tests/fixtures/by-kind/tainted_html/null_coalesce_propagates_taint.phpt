===description===
`$_GET['x'] ?? 'default'` is the single most common way to read a
superglobal defensively, but `is_expr_tainted` had a `Ternary` arm and no
`NullCoalesce` arm, so it fell through to the untainted catch-all and this
extremely common idiom silently bypassed taint tracking entirely.
===config===
suppress=MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    $name = $_GET['name'] ?? 'default';
    echo $name;
}
===expect===
TaintedHtml@4:4-4:15: Tainted HTML output — possible XSS
