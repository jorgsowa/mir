===description===
reports missing static method
===file===
<?php
class Foo {}
function test(): void {
    Foo::missing();
}
===expect===
UndefinedMethod@4:5-4:19: Method Foo::missing() does not exist
