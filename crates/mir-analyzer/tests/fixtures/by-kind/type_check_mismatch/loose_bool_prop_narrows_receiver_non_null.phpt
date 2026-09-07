===description===
Loose `$obj->prop == true` / `true == $obj->prop` narrows the receiver
non-null the same way the strict `===` sibling already does — `null ==
true` is false, so a true match against the literal `true` also proves
the receiver wasn't null. Without it, a nullable receiver's own
nullability leaks back into the property's resolved type even after the
property itself narrowed correctly.
===config===
suppress=UnusedVariable,PossiblyNullPropertyAccess
===file===
<?php
class Box {
    public ?bool $flag = null;
}

function rightOperand(?Box $x): void {
    if ($x->flag == true) {
        /** @mir-check $x->flag is true */
        $_ = 1;
    }
}

function leftOperand(?Box $x): void {
    if (true == $x->flag) {
        /** @mir-check $x->flag is true */
        $_ = 1;
    }
}

function falseMatchDoesNotNarrowNonNull(?Box $x): void {
    if ($x->flag == false) {
        // `null == false` is true, so a false-literal match does NOT
        // prove the receiver non-null — flag's own type still admits null.
        /** @mir-check $x->flag is false|null */
        $_ = 1;
    }
}
===expect===
PossiblyNullPropertyFetch@7:8-7:16: Cannot access property $flag on possibly null value
PossiblyNullPropertyFetch@14:16-14:24: Cannot access property $flag on possibly null value
PossiblyNullPropertyFetch@21:8-21:16: Cannot access property $flag on possibly null value
