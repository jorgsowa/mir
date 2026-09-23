===description===
Value of unit enum
===file===
<?php
    enum Foo
    {
        case Foo;
        case Bar;
    }

    /** @param value-of<Foo> $arg */
    function foobar(string $arg): void {}
===expect===
UnusedParam@9:20-9:31: Parameter $arg is never used
