===description===
!isset short-circuit with || operator — property chain on narrowed variable
Variable used with property access in RHS should be narrowed as defined from !isset() LHS
===file===
<?php
if (!isset($obj) || $obj->prop->method()) {
    // After fix: $obj should be narrowed as defined in RHS
}
===expect===
MixedMethodCall@2:20: Method method() called on mixed type
