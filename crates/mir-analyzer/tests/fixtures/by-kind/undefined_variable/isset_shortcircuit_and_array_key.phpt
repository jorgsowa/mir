===description===
isset short-circuit with && operator — method call on narrowed variable
isset($data) && $data->method() applies narrowing from LHS to method call in RHS
===file===
<?php
if (isset($data) && $data->method()) {
//                  ^^^^^^^^^^^^^^^ MixedMethodCall: Method method() called on mixed type
    /** @mir-check $data is mixed */
}
