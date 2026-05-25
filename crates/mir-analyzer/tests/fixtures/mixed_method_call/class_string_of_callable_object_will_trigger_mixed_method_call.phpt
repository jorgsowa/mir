===description===
Class string of callable object will trigger mixed method call
===file===
<?php
/**
 * @param class-string<object&callable> $className
 */
function takesCallableObject(string $className): void {
    new $className();
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
MixedMethodCall
===ignore===
TODO
