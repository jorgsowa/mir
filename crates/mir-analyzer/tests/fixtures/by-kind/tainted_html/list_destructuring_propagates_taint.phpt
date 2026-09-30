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
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $b;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
