===source===
<?php
class Foo {}
function test(): void {
    Foo::missing();
}
===expect===
UndefinedMethod at 4:4
