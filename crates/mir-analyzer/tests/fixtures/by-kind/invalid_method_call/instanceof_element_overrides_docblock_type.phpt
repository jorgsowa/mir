===description===
`$a[$k] instanceof X` narrows the element to `X` even when the array's docblock element type rules `X` out: the passing check shows the docblock is wrong. A failing check leaves the element type as declared.
===file===
<?php
declare(strict_types=1);

interface Context
{
    public function value(): string;
}

/** @param string[] $contexts */
function contextValue(array $contexts, string $id): string
{
    if (isset($contexts[$id]) && $contexts[$id] instanceof Context) {
        return $contexts[$id]->value();
    }
    if (!$contexts[$id] instanceof Context) {
        $contexts[$id]->value();
    //  ^^^^^^^^^^^^^^^^^^^^^^^ InvalidMethodCall: Cannot call method value() on non-object type 'string'
    }
    return '';
}
