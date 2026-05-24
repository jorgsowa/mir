===description===
noExceptionOnMissingClass
===file===
<?php
/** @psalm-suppress UndefinedClass */
class A
{
    /** @var class-string<Foo> */
    protected $bar;

    public function foo(string $s): void
    {
        $bar = $this->bar;
        $bar::baz();
    }
}
===expect===
MissingConstructor
===ignore===
TODO
