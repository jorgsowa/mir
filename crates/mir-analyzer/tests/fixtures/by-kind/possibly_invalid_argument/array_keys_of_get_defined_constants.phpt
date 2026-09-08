===description===
M7 path A: the get_defined_constants() stub declares its keys as `string`
(`@return array<string, mixed>`), so array_keys() yields `string` keys
and no PossiblyInvalidArgument is reported for string params.
===config===
suppress=ForbiddenCode
===file===
<?php
function dump(): void
{
    foreach (array_keys(get_defined_constants()) as $key) {
        takesString($key);
    }
}

function takesString(string $s): void
{
    var_dump($s);
}
===expect===
