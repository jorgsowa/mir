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
    new namespace\A('y');
}
===expect===
InvalidArgument@14:15-14:18: Argument $n of App\A::__construct() expects 'int', got '"x"'
InvalidArgument@15:20-15:23: Argument $n of App\A::__construct() expects 'int', got '"y"'
