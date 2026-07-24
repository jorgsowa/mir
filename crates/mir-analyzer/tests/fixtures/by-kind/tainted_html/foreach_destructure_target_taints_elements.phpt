===description===
`foreach ($tainted as [$a, $b])` — a destructured (non-plain-variable)
value target falls into `assign_to_target`, which has no taint-marking
of its own, so every destructured element silently dropped taint
entirely, unlike a plain `foreach ($tainted as $v)` value binding.
===config===
suppress=MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $pairs = [[$_GET['id'], $_GET['name']]];
    foreach ($pairs as [$id, $name]) {
        echo $name;
    }
}
===expect===
TaintedHtml@5:8-5:19: Tainted HTML output — possible XSS
