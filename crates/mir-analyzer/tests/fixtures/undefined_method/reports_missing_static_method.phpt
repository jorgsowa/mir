===source===
<?php
class Foo {}
function test(): void {
    Foo::missing();
}
===expect===
UndefinedMethod: Foo::missing()
