===description===
`key-of<T>` preserves caller-inferred literal keys through template calls.
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
