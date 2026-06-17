===description===
wrong argument type is still caught when the class is discovered via PSR-4 lazy loading
===file:composer.json===
{"autoload":{"psr-4":{"Svc\\":"src/"}}}
===file:src/Mailer.php===
<?php
namespace Svc;
class Mailer {
    public function send(string $address): void { var_dump($address); }
}
===file:App.php===
<?php
function run(): void {
    $m = new \Svc\Mailer();
    $m->send(42);
}
===expect===
App.php: ArgumentTypeCoercion@4:13-4:15: Argument $address of send() expects 'string', got '42' — coercion may fail at runtime
