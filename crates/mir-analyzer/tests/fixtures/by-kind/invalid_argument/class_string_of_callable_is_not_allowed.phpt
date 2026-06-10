===description===
Class string of callable is not allowed
===file===
<?php
/**
 * @param class-string<callable():int> $className
 */
function takesCallableObject(string $className): int {
    $object = new $className();
    return $object();
}

class Foo
{
    public function __invoke(): int
    {
        return 0;
    }
}

takesCallableObject(Foo::class);

===expect===
