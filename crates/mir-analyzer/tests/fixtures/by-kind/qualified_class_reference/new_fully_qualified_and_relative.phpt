===description===
`new` resolves fully qualified and `namespace\` relative class names.
===file===
<?php
namespace App;

class A {
    public function __construct(public int $n) {}
}

function ok(): A {
    new \App\A(1);
    return new namespace\A(2);
}

function bad(): void {
    new \App\A('x');
//             ^^^ InvalidArgument: Argument $n of App\A::__construct() expects 'int', got '"x"'
    new namespace\A('y');
//                  ^^^ InvalidArgument: Argument $n of App\A::__construct() expects 'int', got '"y"'
}
