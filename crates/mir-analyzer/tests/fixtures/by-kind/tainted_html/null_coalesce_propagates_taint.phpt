===description===
`$_GET['x'] ?? 'default'` is the single most common way to read a
superglobal defensively, but `is_expr_tainted` had a `Ternary` arm and no
`NullCoalesce` arm, so it fell through to the untainted catch-all and this
extremely common idiom silently bypassed taint tracking entirely.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $name = $_GET['name'] ?? 'default';
    echo $name;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
