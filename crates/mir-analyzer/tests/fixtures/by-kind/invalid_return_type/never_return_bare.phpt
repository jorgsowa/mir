===description===
P2: A bare `return;` inside a `: never`-declared function is a PHP parse error.
Documents that PHP's parse-level enforcement covers bare returns in never functions.
===file===
<?php

function bare_return(): never {
    return;
//  ^^^^^^^ ParseError: Parse error: A never-returning function must not return
}

class Foo {
    public function method_bare_return(): never {
        return;
//      ^^^^^^^ ParseError: Parse error: A never-returning function must not return
    }
}
