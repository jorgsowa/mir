===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Greetable.php===
<?php
namespace App;
trait Greetable {
    public function greet(): string { return 'hi'; }
}
===file:Host.php===
<?php
trait Farewell {
    use \App\Greetable;
    public function bye(): string { return 'bye'; }
}
class Host { use Farewell; }
function test(): void {
    $h = new Host();
    $h->greet();
    $h->bye();
}
===expect===
