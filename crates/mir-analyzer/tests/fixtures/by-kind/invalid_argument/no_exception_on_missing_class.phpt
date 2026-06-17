===description===
No exception on missing class
===config===
suppress=MissingPropertyType,UnusedParam
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
UnusedSuppress@3:0-3:0: Suppress annotation for 'UndefinedClass' is never used
