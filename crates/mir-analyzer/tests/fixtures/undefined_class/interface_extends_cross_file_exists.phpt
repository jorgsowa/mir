===file:Countable.php===
<?php
namespace App;
interface Countable {
    public function count(): int;
}
===file:Collection.php===
<?php
use App\Countable;
interface Collection extends Countable {
    public function isEmpty(): bool;
}
===expect===
