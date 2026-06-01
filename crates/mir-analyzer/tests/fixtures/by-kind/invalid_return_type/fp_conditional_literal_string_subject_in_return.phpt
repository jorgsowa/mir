===description===
FP guard: conditional return with literal-string subject widened when returned as string (Str::studly return type)
===file===
<?php

/**
 * @param string $value
 * @return ($value is "" ? "" : string)
 */
function studly(string $value): string
{
    return $value;
}

function process(string $input): string
{
    // studly() widens to ""| string = string; should NOT fire InvalidReturnType
    return studly($input);
}
===expect===
