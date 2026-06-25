===description===
Dollar-named params inside callable/generic type syntax must not be chosen as the
PHPDoc parameter name. The depth-tracking parser skips $a inside callable(int $a)
and picks $callback at depth 0.
===file===
<?php

/**
 * @param callable(int $a, string $b): bool $callback The callback to invoke
 * @param string $label The label for $callback
 * @suppress UnusedParam
 */
function run($callback, $label): void {
    /** @mir-check $label is string */
}

===expect===
