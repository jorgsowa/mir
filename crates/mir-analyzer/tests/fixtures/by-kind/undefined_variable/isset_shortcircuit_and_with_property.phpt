===description===
isset short-circuit with && operator — property access on narrowed variable
isset($obj) && $obj->prop applies narrowing from LHS to property access in RHS
===config===
suppress=MixedPropertyFetch
===file===
<?php
if (isset($obj) && $obj->prop) {
    // After fix: $obj should be narrowed as defined in RHS
}
===expect===
