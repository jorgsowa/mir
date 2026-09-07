===description===
`is_subclass_of($x, Interface::class)` on an interface-typed value forms
an intersection instead of dropping the original type when the two
interfaces are unrelated but a single object could implement both —
mirrors instanceof's existing classes_can_coexist fallback, which
narrow_strict_subclass_of lacked. Two unrelated concrete classes remain
mutually exclusive and are still correctly dropped.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Nameable {}
interface Greeter {}
class ConcreteA {}
class ConcreteB {}

function preservesCoexistingInterface(Nameable $x): void {
    if (is_subclass_of($x, Greeter::class)) {
        /** @mir-check $x is Nameable&Greeter */
        $_ = 1;
    }
}

function unrelatedConcreteClassesStillDropped(ConcreteA $x): void {
    if (is_subclass_of($x, ConcreteB::class)) {
        // Narrowing to empty leaves $x unchanged (mark_diverges=false: being
        // false for the exact class doesn't make this branch unreachable).
        /** @mir-check $x is ConcreteA */
        $_ = 1;
    }
}
===expect===
