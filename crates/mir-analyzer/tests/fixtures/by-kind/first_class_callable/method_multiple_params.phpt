===description===
P3: Method first-class callable preserves all parameter types in the Closure signature.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Formatter {
    public function format(string $template, int $value, bool $pad): string {
        return sprintf($template, $value);
    }
}

$f = new Formatter();
$fn = $f->format(...);
/** @mir-check $fn is Closure(string, int, bool): string */
$_ = $fn;
===expect===
