===description===
P22: A `@param $name description` line with no type (valid PHPDoc grammar — the type
is optional) must not be parsed as if the whole body were a type expression. The body
starts directly with `$name`, so `parse_param_line`'s whitespace-before-`$name` scan
never matches; the docblock's variable name was falling into the fallback full-body
type validation and getting flagged as a variable in type position.
===config===
suppress=MissingReturnType
===file===
<?php

/** @param $skipUncloneable whether to skip uncloneable objects */
function f($skipUncloneable): void {
    /** @mir-check $skipUncloneable is mixed */
    echo $skipUncloneable;
}
===expect===
