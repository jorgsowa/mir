===description===
tap() where first arg is ob_get_contents() (string|false) and declared return is string.
In non-strict PHP, string|false → string is a scalar coercion, not InvalidReturnType.
===config===
suppress=MixedArgument,MixedAssignment,UnusedParam,MissingReturnType,MissingClosureReturnType
===file===
<?php

class HigherOrderTapProxy {}

/**
 * @template TValue
 * @param TValue $value
 * @param (callable(TValue): mixed)|null $callback
 * @return ($callback is null ? HigherOrderTapProxy : TValue)
 */
function tap($value, $callback = null) {
    if (is_null($callback)) {
        return new HigherOrderTapProxy();
    }
    $callback($value);
    return $value;
}

function renderView(): string {
    ob_start();
    echo "hello";
    return tap(ob_get_contents(), function () {
        ob_end_clean();
    });
}
===expect===
