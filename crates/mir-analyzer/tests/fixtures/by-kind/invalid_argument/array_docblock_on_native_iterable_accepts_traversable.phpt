===description===
`T[]` on a native `iterable` parameter reads as `iterable<T>`: a `Traversable` argument is accepted, on a function and on a promoted constructor parameter, while its element type is still checked.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

interface Factory {}

final class Transport
{
    /** @param Factory[] $factories */
    public function __construct(private iterable $factories) {}

    /** @return \Traversable<int, Factory> */
    public static function defaults(): \Traversable
    {
        return new \ArrayIterator([]);
    }
}

/** @param Factory[] $factories */
function register(iterable $factories): void
{
    foreach ($factories as $factory) {
        /** @mir-check $factory is Factory */
        echo get_class($factory);
    }
}

/** @param \ArrayIterator<int, string> $names */
function run(\ArrayIterator $names): void
{
    new Transport(Transport::defaults());
    register(Transport::defaults());
    register([]);
    register($names);
    //       ^^^^^^ InvalidArgument: Argument $factories of register() expects 'iterable<int|string, Factory>', got 'ArrayIterator<int, string>'
}
