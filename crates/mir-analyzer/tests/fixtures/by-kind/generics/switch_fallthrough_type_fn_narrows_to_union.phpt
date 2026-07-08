===description===
FP: a `switch(true)` fallthrough over scalar type-check functions
(`is_int($x)`, `is_string($x)`) must narrow to the union of both, not just
the last label's type — narrow_instanceof_disjuncts only recognized
`instanceof` conditions, so this shape fell through to narrowing each
condition individually (AND semantics), collapsing to the last one.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param int|string $x
 */
function bar($x): void {
    switch (true) {
        case is_int($x):
        case is_string($x):
            /** @mir-check $x is int|string */
            $_ = 1;
            break;
    }
}
===expect===
