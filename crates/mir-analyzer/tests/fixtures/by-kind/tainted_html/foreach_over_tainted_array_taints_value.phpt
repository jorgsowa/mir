===description===
`foreach ($_POST as $v) { echo $v; }` is an extremely common way to handle
form data, but the foreach analyzer bound the loop variable's TYPE without
ever calling taint_var, so the loop value silently bypassed taint tracking
regardless of whether the iterated collection was tainted.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    foreach ($_POST as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
