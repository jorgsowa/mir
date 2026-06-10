===description===
No exception on missing class
===ignore===
TODO
===file===
<?php
/** @suppress UndefinedClass */
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
