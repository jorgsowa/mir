===description===
No NullArgument when parameter type is an unbound template variable (null is a valid binding)
===file===
<?php

/**
 * @template R
 * @param R $value
 * @return R
 */
function success($value) { return $value; }

/**
 * @template T
 * @param T $value
 * @return T
 */
function some($value) { return $value; }

success(null);
some(null);
===expect===
