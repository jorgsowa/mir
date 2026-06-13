===description===
UndefinedDocblockClass does NOT fire for template type parameters — @template T
introduces T as a valid type name for the function's scope.
===file===
<?php
/**
 * @template T
 * @param T $item
 * @return T
 */
function identity(mixed $item): mixed {
    return $item;
}

===expect===
