===file:Collection.php===
<?php
use App\Countable;
interface Collection extends Countable {
    public function isEmpty(): bool;
}
===expect===
Collection.php: UndefinedClass: Class App\Countable does not exist
