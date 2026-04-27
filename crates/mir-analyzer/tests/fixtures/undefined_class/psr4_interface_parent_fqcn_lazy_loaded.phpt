===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Countable.php===
<?php
namespace App;
interface Countable {
    public function count(): int;
}
===file:Collection.php===
<?php
interface Collection extends \App\Countable {
    public function isEmpty(): bool;
}
===expect===
