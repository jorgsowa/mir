===description===
`!is_array($x) || !array_is_list($x)` on a `mixed`-typed variable must not
be flagged as always true/false. `is_array($x)` narrows an unknown key to
`Type::mixed()`, not a bare `TInt`, and `array_is_list`'s narrowing used to
only recognize a literal int key — the mismatch made the combined
false-branch narrowing collapse to an impossible type, which falsely
proved the whole condition always true.
===file===
<?php
function isList(mixed $array): array
{
    if (!is_array($array) || !array_is_list($array)) {
        throw new \InvalidArgumentException();
    }

    return $array;
}
===expect===
