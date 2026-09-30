===description===
reports missing static method
===file===
<?php
class Foo {}
function test(): void {
    Foo::missing();
//  ^^^^^^^^^^^^^^ UndefinedMethod: Method Foo::missing() does not exist
}
===expect===
