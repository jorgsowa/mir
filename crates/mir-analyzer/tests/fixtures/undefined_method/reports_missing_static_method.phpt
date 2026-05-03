===description===
reports missing static method
===file===
<?php
class Foo {}
function test(): void {
    Foo::missing();
}
===expect===
UndefinedMethod: Method Foo::missing() does not exist
===ignore===
TODO
