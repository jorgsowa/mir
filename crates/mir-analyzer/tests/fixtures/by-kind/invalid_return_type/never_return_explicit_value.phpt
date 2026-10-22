===description===
Returning a value from a `: never` function is an error.
===file===
<?php

function returns_string(): never {
    return "hello";
//  ^^^^^^^^^^^^^^^ InvalidReturnType: Return type '"hello"' is not compatible with declared 'never'
}

function returns_int(): never {
    return 42;
//  ^^^^^^^^^^ InvalidReturnType: Return type '42' is not compatible with declared 'never'
}

function returns_null(): never {
    return null;
//  ^^^^^^^^^^^^ InvalidReturnType: Return type 'null' is not compatible with declared 'never'
}

class Foo {
    public function method_returns_value(): never {
        return true;
//      ^^^^^^^^^^^^ InvalidReturnType: Return type 'true' is not compatible with declared 'never'
    }
}
