===description===
`Foo::class` in a constant initializer is a class-string, so the constant array keeps its shape.
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

const DATETIME_CLASSES = [
    'immutable' => \DateTimeImmutable::class,
    'mutable' => \DateTime::class,
];

final class Registry
{
    public const DEFAULT = \ArrayObject::class;
}

/**
 * @template T of key-of<DATETIME_CLASSES>
 * @param T $type
 */
function createNow(string $type): \DateTimeInterface
{
    return new (DATETIME_CLASSES[$type]);
}

function takesInt(int $value): void {}

createNow('immutable');
createNow('mutable');
createNow('other');
//<^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $type of createNow() expects 'key-of<App\DATETIME_CLASSES>', got '"other"'
takesInt(Registry::DEFAULT);
//       ^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'class-string<ArrayObject>'
