===description===
`key-of<T>` over a function template should stay tied to the caller's inferred
array keys. Falling back to plain `mixed` loses both parts of the contract:
the foreach value binding in the callee spuriously reports MixedAssignment, and
valid callers passing the returned key to `string` or a literal-key union
spuriously report MixedArgument.
===config===
suppress=UnusedParam,UnusedForeachValue
===file===
<?php

/**
 * @template T of array<array-key, mixed>
 * @param T $items
 * @return key-of<T>
 */
function firstKey(array $items)
{
    foreach ($items as $key => $_) {
        return $key;
    }

    throw new \InvalidArgumentException('empty array');
}

function acceptsString(string $value): void
{
}

/**
 * @param 'debug'|'verbose' $flag
 */
function acceptsFlagName(string $flag): void
{
}

acceptsString(firstKey(['debug' => false, 'verbose' => true]));
acceptsFlagName(firstKey(['debug' => false, 'verbose' => true]));
===expect===
