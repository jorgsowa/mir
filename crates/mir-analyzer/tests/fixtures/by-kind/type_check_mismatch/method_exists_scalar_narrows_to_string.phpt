===description===
method_exists()/property_exists() true branch narrows a `bool|int|float|string`
argument to `string`, not `object` — a scalar can never be an object instance.
===config===
suppress=UnusedVariable,UnusedParam,MixedMethodCall
===file===
<?php
/** @param scalar $x */
function test_method_exists_scalar_narrows_to_string($x): void {
    if (method_exists($x, 'bar')) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

/** @param scalar $x */
function test_property_exists_scalar_narrows_to_string($x): void {
    if (property_exists($x, 'prop')) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}
===expect===
