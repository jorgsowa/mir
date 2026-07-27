===description===
`@$_GET['x']` (the error-suppression operator, silencing an undefined-
array-key notice) is an extremely common defensive idiom, but
`is_expr_tainted` had no `ErrorSuppress` arm and fell through to the
untainted catch-all — the `@` operator only silences a notice, it
doesn't sanitize the value.
===config===
suppress=MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    $name = @$_GET['name'];
    echo $name;
}
===expect===
TaintedHtml@4:4-4:15: Tainted HTML output — possible XSS
