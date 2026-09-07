===description===
P22 (composite forms): the type-omitted `@param $name` form must also resolve when
`$name` is prefixed with `&` (by-reference) or `...` (variadic) — both prefixes must be
stripped before checking for the leading `$`, same as the typed-prefix path already does
for the whitespace-preceded case.
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php

/** @param &$ref no type, by-reference */
function g(&$ref): void {}

/** @param ...$rest no type, variadic */
function h(...$rest): void {}
===expect===
