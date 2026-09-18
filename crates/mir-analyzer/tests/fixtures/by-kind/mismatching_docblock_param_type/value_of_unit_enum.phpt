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
'
===expect===
ParseError@10:0-12:98: Parse error: unterminated string literal
ParseError@12:98-12:98: Parse error: expected ';' after expression
