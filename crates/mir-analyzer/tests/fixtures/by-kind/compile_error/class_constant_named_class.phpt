===description===
A class constant cannot be named `class`.
===file===
<?php
class Foo
{
    /** @var class-string<Bar> */
    protected const CLASS = Bar::class;
}

class Bar {}
===expect===
CompileError@5:20-5:25: Compile error: A class constant must not be called 'class'; it is reserved for class name fetching [class-constant-named-class]
