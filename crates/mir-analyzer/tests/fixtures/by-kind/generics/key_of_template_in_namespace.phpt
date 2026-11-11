===description===
`key-of<T>` in a namespaced file names the template, and resolves to the keys of the argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

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

/** @param 'debug'|'verbose' $flag */
function acceptsFlagName(string $flag): void {}

function acceptsString(string $value): void {}

function acceptsInt(int $value): void {}

acceptsString(firstKey(['debug' => false, 'verbose' => true]));
acceptsFlagName(firstKey(['debug' => false, 'verbose' => true]));
acceptsInt(firstKey(['debug' => false, 'verbose' => true]));
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsInt() expects 'int', got '"debug"|"verbose"'
