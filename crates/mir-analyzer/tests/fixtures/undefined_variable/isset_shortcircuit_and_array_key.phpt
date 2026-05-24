===description===
isset short-circuit with && operator — method call on narrowed variable
isset($data) && $data->method() applies narrowing from LHS to method call in RHS
===file===
<?php
if (isset($data) && $data->method()) {
    // After fix: $data should be narrowed as defined in RHS
}
===expect===
MixedMethodCall@2:20: Method method() called on mixed type
