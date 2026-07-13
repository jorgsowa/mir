===description===
Sibling of callable_docblock_type_mismatch_argument: a matching argument
type must stay silent.
===file===
<?php
/**
 * @param callable(int):void $fn
 */
function apply(callable $fn): void {
    $fn(1);
}
===expect===
