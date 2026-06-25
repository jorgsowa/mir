===description===
An array callable [$obj, 'method'] passed to a callable(string):string typed
parameter must not emit InvalidArgument. The method signature is not statically
resolved for array callables, so the arity check bails out gracefully.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Formatter {
    public function format(string $s): string {
        return strtoupper($s);
    }
}

/** @param callable(string):string $fn */
function applyFormatter(callable $fn, string $value): string {
    return $fn($value);
}

$f = new Formatter();
$result = applyFormatter([$f, 'format'], 'hello');
===expect===
