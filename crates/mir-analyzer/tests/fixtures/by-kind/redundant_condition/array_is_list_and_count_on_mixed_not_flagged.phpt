===description===
`count($x) > 0 && array_is_list($x)` on a variable already narrowed from
`mixed` to `array` (key type `Type::mixed()`, not a bare `TInt`) must not
be flagged as always true/false — same root cause as the OR-form case in
array_is_list_after_is_array_on_mixed_not_flagged.phpt, hit through the
`&&` narrowing path instead of `||`.
===file===
<?php
function isMap(mixed $array): array
{
    if (!is_array($array)) {
        throw new \InvalidArgumentException();
    }

    if (\count($array) > 0 && \array_is_list($array)) {
        throw new \InvalidArgumentException();
    }

    return $array;
}
===expect===
