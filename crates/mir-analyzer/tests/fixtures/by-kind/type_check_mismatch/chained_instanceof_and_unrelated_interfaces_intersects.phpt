===description===
`$x instanceof A && $x instanceof B` for two unrelated interfaces narrows
to the intersection A&B, not just B — the second instanceof used to wipe
out the first's narrowing entirely
===config===
suppress=UnusedParam
===file===
<?php
interface A {}
interface B {}

function f(object $x): void {
    if ($x instanceof A && $x instanceof B) {
        /** @mir-check $x is A&B */
        echo get_class($x);
    }
}
===expect===
