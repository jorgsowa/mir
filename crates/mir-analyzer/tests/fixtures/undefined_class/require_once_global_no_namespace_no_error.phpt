===file:Foo.php===
<?php
class Foo {
    public function hello(): string {
        return 'hi';
    }
}
===file:Main.php===
<?php
require_once __DIR__ . '/Foo.php';
function run(): void {
    new Foo();
}
===expect===
