===description===
reports missing static method
===file===
<?php
class Foo {}
function test(): void {
    Foo::missing();
}
===expect===
UndefinedMethod@4:4-4:18: Method Foo::missing() does not exist
