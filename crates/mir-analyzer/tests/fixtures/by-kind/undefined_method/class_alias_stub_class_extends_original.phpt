===description===
A class declared with the same name as a `class_alias()` target is only an IDE
stub (usually inside `if (false)`), so it inherits the aliased class's members.
===file:Command.php===
<?php
namespace Bridge;

class CommandForV9 {
    public function run(): int { return 0; }
}

class_alias('Bridge\CommandForV9', 'Bridge\Command');

if (false) {
    class Command {}
}
===file:Listener.php===
<?php
namespace Bridge;

class ListenerForV9 {
    public function startTest(): void {}
}

if (\PHP_VERSION_ID >= 80000) {
    class_alias(ListenerForV9::class, Listener::class);
}

if (\false) {
    class Listener {}
}
===file:Use.php===
<?php
namespace App;

use Bridge\Command;
use Bridge\Listener;

function run(Command $c, Listener $l, object $o): int {
    $l->startTest();
    if ($o instanceof Command) {
        $o->missing();
//      ^^^^^^^^^^^^^ UndefinedMethod: Method Bridge\Command::missing() does not exist
    }
    return $c->run();
}
