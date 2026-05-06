===description===
defined class constant not reported
===file===
<?php
class Foo {
    const BAR = 1;
}
function test(): void {
    echo Foo::BAR;
}
===expect===
