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
//                  ^^^^^^^^^^^ UnusedParam: Parameter $arg is never used
