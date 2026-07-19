===description===
FP: declared_return_has_template only checked an array atom's value type, not
its key — a template appearing only as the array key (`array<TKey, string>`)
was invisible to the leniency check that suppresses InvalidReturnType for an
unresolved template.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @template TKey of array-key
 * @return array<TKey, int>
 */
function makeMap(): array
{
    // Value type (int vs string) would normally fail the structural array
    // check — only the unresolved TKey in the key position should suppress
    // InvalidReturnType here, exactly like an unresolved template value would.
    /** @var array<int|string, string> $m */
    $m = [];
    return $m;
}
===expect===
