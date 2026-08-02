===description===
A bare `@param Closure` inside a namespace must stay resolved to the real
global `Closure` class, not get namespace-qualified — regression guard for
the same-namespace class-name qualification fix, which now qualifies most
bare docblock class names but must still exempt real global builtins.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
namespace App;

/**
 * @param Closure $callback
 */
function apply($callback): void {
}
===expect===
