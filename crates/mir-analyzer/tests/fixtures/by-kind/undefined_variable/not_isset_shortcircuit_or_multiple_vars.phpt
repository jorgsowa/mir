===description===
!isset short-circuit with || operator — nested multiple variables
Each variable narrowed independently based on its !isset() check in nested conditions
===file===
<?php
if (!isset($x) || (!isset($y) || ($x->foo() && $y->bar()))) {
    // Should not error: $x and $y are both defined in their respective branches
}
===expect===
MixedMethodCall@2:35-2:44: Method foo() called on mixed type
MixedMethodCall@2:48-2:57: Method bar() called on mixed type
