===description===
isset short-circuit with && operator — method call on narrowed variable
isset($data) && $data->method() applies narrowing from LHS to method call in RHS
===file===
<?php
if (isset($data) && $data->method()) {
    /** @mir-check $data is mixed */
}
===expect===
MixedMethodCall@2:21-2:36: Method method() called on mixed type
