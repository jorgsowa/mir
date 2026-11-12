===description===
`new<…>` and offset access `CONST[T]` in `@return` are not class names.
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

const ID_TABLE = [
    'immutable' => 1,
    'mutable' => 'two',
];

/**
 * @template T of key-of<ID_TABLE>
 * @param T $type
 * @return ID_TABLE[T]
 */
function lookUp(string $type): int|string
{
    return ID_TABLE[$type];
}

/**
 * @template TClass of string
 * @param TClass $class
 * @return new<TClass>
 */
function instantiate(string $class): object
{
    return new $class();
}

function takesObject(object $value): void {}

function takesArray(array $value): void {}

takesObject(instantiate(\DateTimeImmutable::class));
takesArray(instantiate(\DateTimeImmutable::class));
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesArray() expects 'array', got 'DateTimeImmutable'
takesArray(lookUp('immutable'));
//         ^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesArray() expects 'array', got '1'
