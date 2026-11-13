===description===
`print_r()` and `var_export()` with `$return = true` return the dump instead of printing it.
===file===
<?php
/** @psalm-pure */
function dump(mixed $v): string {
    return print_r($v, true) . var_export($v, true);
}

/** @psalm-pure */
function show(mixed $v): void {
    print_r($v);
//  ^^^^^^^^^^^ ImpureFunctionCall: Calling impure function print_r() in a @pure function
}
