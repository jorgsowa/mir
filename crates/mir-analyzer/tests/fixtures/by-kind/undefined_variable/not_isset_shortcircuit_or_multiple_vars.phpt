===description===
!isset short-circuit with || operator — nested multiple variables
Each variable narrowed independently based on its !isset() check in nested conditions
===file===
<?php
if (!isset($x) || (!isset($y) || ($x->foo() && $y->bar()))) {
//                                ^^^^^^^^^ MixedMethodCall: Method foo() called on mixed type
//                                             ^^^^^^^^^ MixedMethodCall: Method bar() called on mixed type
    // Should not error: $x and $y are both defined in their respective branches
}
