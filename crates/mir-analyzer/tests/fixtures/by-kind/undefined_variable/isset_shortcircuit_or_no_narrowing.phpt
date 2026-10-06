===description===
isset short-circuit with || operator — correctly reports error (control case)
isset($x) || $x->method() should error: RHS only executes when isset($x) is false
===file===
<?php
if (isset($x) || $x->method()) {
//               ^^^^^^^^^^^^ MixedMethodCall: Method method() called on mixed type
//               ^^ UndefinedVariable: Variable $x is not defined
    // Correctly should error: RHS runs when isset($x) is FALSE, so $x is undefined
}
