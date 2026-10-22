===description===
A bare `return;` inside a `: never` function is an error.
===file===
<?php

function bare_return(): never {
    return;
//  ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'never'
}

class Foo {
    public function method_bare_return(): never {
        return;
//      ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'never'
    }
}
