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
UndefinedConstant@6:9-6:21: Constant Foo::MISSING is not defined
