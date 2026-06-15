===description===
!isset short-circuit with || operator — multiple uses of variable in RHS
Ensures narrowing applies to all uses of the variable within the RHS expression
===file===
<?php
if (!isset($x) || ($x->foo() && $x->bar())) {
    // After fix: no UndefinedVariable errors for $x in RHS of !isset($x) ||
}
===expect===
MixedMethodCall@2:19-2:28: Method foo() called on mixed type
MixedMethodCall@2:32-2:41: Method bar() called on mixed type
