===description===
isset short-circuit with && operator — variable narrowed as defined in RHS
isset($x) && use($x) applies narrowing from LHS to RHS in short-circuit evaluation
===file===
<?php
if (isset($x) && $x->method()) {
    // After fix: no UndefinedVariable on RHS of isset($x) &&
}
===expect===
MixedMethodCall@2:18-2:30: Method method() called on mixed type
