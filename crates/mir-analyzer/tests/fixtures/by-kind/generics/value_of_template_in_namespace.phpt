===description===
`value-of<T>` in a namespaced file names the template, and resolves to the values of the argument.
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
 * @return value-of<T>
 */
function firstValue(array $items)
{
    foreach ($items as $value) {
        return $value;
    }

    throw new \InvalidArgumentException('empty array');
}

/** @param 1|10 $level */
function acceptsLevel(int $level): void {}

function acceptsInt(int $value): void {}

function acceptsString(string $value): void {}

acceptsInt(firstValue(['low' => 1, 'high' => 10]));
acceptsLevel(firstValue(['low' => 1, 'high' => 10]));
acceptsString(firstValue(['low' => 1, 'high' => 10]));
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $value of acceptsString() expects 'string', got '1|10' — coercion may fail at runtime
