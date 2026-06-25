===description===
@psalm-type alias defined on a method docblock does not produce false positives
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class EventDispatcher {
    /**
     * @psalm-type ListenerConfig = array{event: string, handler: callable}
     * @param ListenerConfig $config
     * @return void
     */
    public function register(array $config): void {}
}

$d = new EventDispatcher();
$d->register(['event' => 'login', 'handler' => fn() => null]);
===expect===
