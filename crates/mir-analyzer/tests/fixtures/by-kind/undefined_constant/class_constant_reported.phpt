===description===
class constant reported
===file===
<?php
class Foo {
    const REAL = 1;
}
function test(): void {
    echo Foo::MISSING;
}
===expect===
UndefinedConstant@6:10-6:22: Constant Foo::MISSING is not defined
