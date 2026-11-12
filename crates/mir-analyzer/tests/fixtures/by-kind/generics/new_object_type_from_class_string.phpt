===description===
`@return new<TClass>` is an instance of the class the caller passed; an unknown class-string gives `object`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

function takesImmutable(\DateTimeImmutable $value): void {}

/**
 * @template TClass of string
 * @param TClass $class
 * @return new<TClass>
 */
function instantiate(string $class): object
{
    return new $class();
}

function test(string $class): void
{
    takesImmutable(instantiate(\DateTimeImmutable::class));
    takesImmutable(instantiate(\DateTime::class));
    //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesImmutable() expects 'DateTimeImmutable', got 'DateTime'
    $unknown = instantiate($class);
    /** @mir-check $unknown is object */
    echo $unknown::class;
}
