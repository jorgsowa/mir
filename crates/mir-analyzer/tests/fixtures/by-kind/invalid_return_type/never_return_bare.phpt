===description===
P2: A bare `return;` inside a `: never`-declared function is a PHP parse error.
Documents that PHP's parse-level enforcement covers bare returns in never functions.
===file===
<?php

function bare_return(): never {
    return;
}

class Foo {
    public function method_bare_return(): never {
        return;
    }
}
===expect===
ParseError@4:4-4:11: Parse error: A never-returning function must not return
ParseError@9:8-9:15: Parse error: A never-returning function must not return
