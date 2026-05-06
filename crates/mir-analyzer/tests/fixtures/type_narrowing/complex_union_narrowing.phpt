===description===
Complex union type narrowing scenarios
===file===
<?php
function multiUnionNarrowing(int|string|null|bool $value) {
    if (!is_null($value)) {
        // Now: int|string|bool
        if (is_string($value)) {
            strlen($value);
        } else if (is_int($value)) {
            $value + 1;
        } else {
            var_dump($value);
        }
    }
}

function nestedUnionNarrowing(int|string|float|null|bool $x) {
    if ($x === null) {
        return null;
    }
    // Now: int|string|float|bool

    if (!is_string($x)) {
        if (!is_int($x)) {
            if (!is_float($x)) {
                var_dump($x);
            }
        }
    }
}

function multiNullRemoval(int|float|string|null|bool|array $value) {
    if ($value !== null) {
        // int|float|string|bool|array
        return strlen((string) $value);
    }
}
===expect===
InvalidCast@33:31: Cannot cast 'int|float|string|bool|array<mixed, mixed>' to 'string'
