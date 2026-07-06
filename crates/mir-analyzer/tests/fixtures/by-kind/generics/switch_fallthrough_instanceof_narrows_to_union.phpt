===description===
FP: a `switch(true)` case label with an empty body falls through into the
next case's body — the shared body executes whenever EITHER label's
condition matched, OR semantics, so $foo must narrow to A|B there, not just
the last label's class (B). Each case started narrowing from a fresh branch
of the pre-switch state, discarding the earlier fallen-through label's
narrowing entirely.
===config===
suppress=UnusedVariable
===file===
<?php
interface Foo {}
class A implements Foo {}
class B implements Foo {}

function bar(Foo $foo): void {
    switch (true) {
        case $foo instanceof A:
        case $foo instanceof B:
            /** @mir-check $foo is A|B */
            $_ = 1;
            break;
    }
}

function bar_literal(int $x): void {
    switch ($x) {
        case 1:
        case 2:
            /** @mir-check $x is 1|2 */
            $_ = 1;
            break;
    }
}
===expect===
