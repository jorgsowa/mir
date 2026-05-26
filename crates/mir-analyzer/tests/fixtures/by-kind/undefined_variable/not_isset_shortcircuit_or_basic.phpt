===description===
!isset short-circuit with || operator — variable narrowed as defined in RHS
Classic PHP idiom: !isset($x) || use($x) should not error on UndefinedVariable in RHS
===file===
<?php
if (!isset($x) || $x->method()) {
    // After fix: should NOT error on UndefinedVariable
    // If !isset($x) is false, then $x IS defined
}
===expect===
MixedMethodCall@2:19: Method method() called on mixed type
