===description===
`ReflectionClass::newInstanceArgs()` returns the reflected class instance, not null: PHP 8 throws instead of returning null. A subclass may still declare the native `?object` return.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

final class Point
{
    public function __construct(public int $x) {}
}

function make(): Point
{
    $point = (new ReflectionClass(Point::class))->newInstanceArgs([1]);
    /** @mir-check $point is Point */
    return $point;
}

/** @param class-string $class */
function build(string $class): object
{
    return (new ReflectionClass($class))->newInstanceArgs([]);
}

/** @extends ReflectionClass<object> */
final class StaticReflection extends ReflectionClass
{
    public function newInstanceArgs(array $args = []): ?object
    {
        return null;
    }
}
