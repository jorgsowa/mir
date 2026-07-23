===description===
`[$a, $b] = $_GET['pair'];` had no taint propagation at all — plain
variable/property assignment both taint their target, but array
destructuring was the one assignment-target shape with no equivalent.
===config===
suppress=MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    [$a, $b] = $_GET['pair'];
    echo $a;
    echo $b;
}
===expect===
TaintedHtml@4:4-4:12: Tainted HTML output — possible XSS
TaintedHtml@5:4-5:12: Tainted HTML output — possible XSS
