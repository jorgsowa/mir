===description===
cross file psr4 lazy loaded parent method
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Base.php===
<?php
namespace App;
class Base {
    public function hello(): void {}
}
===file:Child.php===
<?php
class Child extends \App\Base {}
function test(): void {
    $c = new Child();
    $c->missing();
}
===expect===
Child.php: UndefinedMethod@5:5-5:18: Method Child::missing() does not exist
