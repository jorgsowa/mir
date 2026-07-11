===description===
A `switch(true)` fallthrough mixing an `instanceof` label with a scalar
type-check function (`case $x instanceof A:` / `case is_string($x):`) must
narrow to the union of both, the same as an all-instanceof or all-type-fn
fallthrough already does.
===config===
suppress=UnusedVariable
===file===
<?php
class A {}
/**
 * @param A|string|int $x
 */
function bar($x): void {
    switch (true) {
        case $x instanceof A:
        case is_string($x):
            /** @mir-check $x is A|string */
            $_ = 1;
            break;
    }
}
===expect===
