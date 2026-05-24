===description===
new with class-string variable should not error
===file===
<?php
class Foo {}

function test(string $className) {
    /** @var class-string<Foo> $className */
    new $className();
}
===expect===
