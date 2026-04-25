===file===
<?php
class Foo {
    const REAL = 1;
}
function test(): void {
    echo Foo::MISSING;
}
===expect===
UndefinedConstant: Constant Foo::MISSING is not defined
