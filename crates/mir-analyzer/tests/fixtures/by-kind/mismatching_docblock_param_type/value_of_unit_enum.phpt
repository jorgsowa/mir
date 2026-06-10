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
// TODO turn this into an InvalidDocblock with a better error message. This is difficult because it
// has to happen after scanning has finished, otherwise the class might not have been scanned yet.
===expect===
ParseError@10:1-12:99: Parse error: unterminated string literal
ParseError@12:99-12:99: Parse error: expected ';' after expression
