===description===
`foreach ($tainted as [$a, $b])` — a destructured (non-plain-variable)
value target falls into `assign_to_target`, which has no taint-marking
of its own, so every destructured element silently dropped taint
entirely, unlike a plain `foreach ($tainted as $v)` value binding.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $pairs = [[$_GET['id'], $_GET['name']]];
    foreach ($pairs as [$id, $name]) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
