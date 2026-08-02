===description===
P20: A docblock-union param (`UnitEnum|string|int|null`, no native type) narrowed via
`instanceof UnitEnum` must resolve `->name` — the interface's own native property, not a
docblock-declared one. Matches egulias-email-validator's `Warning\QuotedPart` constructor.
===file===
<?php

/**
 * @param UnitEnum|string|int|null $token
 */
function describe($token): string
{
    if ($token instanceof UnitEnum) {
        return $token->name;
    }
    return (string) $token;
}
===expect===
