===description===
`!isset($x) || $y instanceof Foo` is true via two disjoint paths: $x unset (says
nothing about $y), or $x set AND $y instanceof Foo. Narrowing $y to Foo must not
leak into the merged true-branch, since $y stays Foo|Bar on the "$x unset" path —
narrowing it down to plain Foo there would mask a genuine error if the branch body
went on to call a Bar-only method assuming $y could never be Bar.
===config===
suppress=UnusedVariable
===file===
<?php
final class Foo {}
final class Bar {}

/** @param Foo|Bar $y */
function test(?int $x, $y): void {
    if (!isset($x) || $y instanceof Foo) {
        /** @mir-check $y is Foo|Bar */
        $_ = $y;
    }
}
===expect===
