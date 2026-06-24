===description===
P2: Returning a value from a `: never`-declared function is a PHP parse error —
the parser already rejects `return $expr;` inside a `: never` body. This fixture
documents that PHP's own parse-level enforcement is in place (not just a
static-analysis check).
===file===
<?php

function returns_string(): never {
    return "hello";
}

function returns_int(): never {
    return 42;
}

function returns_null(): never {
    return null;
}

class Foo {
    public function method_returns_value(): never {
        return true;
    }
}
===expect===
ParseError@4:4-4:19: Parse error: A never-returning function must not return
ParseError@8:4-8:14: Parse error: A never-returning function must not return
ParseError@12:4-12:16: Parse error: A never-returning function must not return
ParseError@17:8-17:20: Parse error: A never-returning function must not return
