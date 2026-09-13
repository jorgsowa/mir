===description===
`array_filter(array_keys(get_defined_constants()))` keeps string values.
===config===
suppress=ForbiddenCode
===file===
<?php
function dump(): void
{
    foreach (array_filter(array_keys(get_defined_constants())) as $errorConstant) {
        takesString($errorConstant);
    }
}

function takesString(string $s): void
{
    var_dump($s);
}
===expect===
