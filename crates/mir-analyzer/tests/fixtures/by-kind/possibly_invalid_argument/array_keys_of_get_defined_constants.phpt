===description===
`array_keys(get_defined_constants())` returns string keys.
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
