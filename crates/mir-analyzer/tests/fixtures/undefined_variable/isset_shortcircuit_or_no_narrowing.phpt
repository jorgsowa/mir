===description===
isset short-circuit with || operator — correctly reports error (control case)
isset($x) || $x->method() should error: RHS only executes when isset($x) is false
===file===
<?php
if (isset($x) || $x->method()) {
    // Correctly should error: RHS runs when isset($x) is FALSE, so $x is undefined
}
===expect===
MixedMethodCall@2:17: Method method() called on mixed type
UndefinedVariable@2:17: Variable $x is not defined
