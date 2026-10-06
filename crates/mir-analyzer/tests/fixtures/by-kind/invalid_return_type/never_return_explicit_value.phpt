===description===
P2: Returning a value from a `: never`-declared function is a PHP parse error —
the parser already rejects `return $expr;` inside a `: never` body. This fixture
documents that PHP's own parse-level enforcement is in place (not just a
static-analysis check).
===file===
<?php

function returns_string(): never {
    return "hello";
//  ^^^^^^^^^^^^^^^ ParseError: Parse error: A never-returning function must not return
}

function returns_int(): never {
    return 42;
//  ^^^^^^^^^^ ParseError: Parse error: A never-returning function must not return
}

function returns_null(): never {
    return null;
//  ^^^^^^^^^^^^ ParseError: Parse error: A never-returning function must not return
}

class Foo {
    public function method_returns_value(): never {
        return true;
//      ^^^^^^^^^^^^ ParseError: Parse error: A never-returning function must not return
    }
}
