===description===
FP guard: conditional return with literal-string subject widened when passed to string param (Str::studly pattern)
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

/** @param string $str */
function takesString(string $str): void { echo $str; }

/** @var string $s */
$s = 'hello';
$result = studly($s);
takesString($result);
===expect===
