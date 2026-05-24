===description===
interface extends cross file missing
===file:Collection.php===
<?php
use App\Countable;
interface Collection extends Countable {
    public function isEmpty(): bool;
}
===expect===
Collection.php: UndefinedClass@3:30: Class App\Countable does not exist
