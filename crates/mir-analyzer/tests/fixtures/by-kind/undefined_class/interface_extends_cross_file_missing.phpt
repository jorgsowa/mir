===description===
interface extends cross file missing
===file:Collection.php===
<?php
use App\Countable;
interface Collection extends Countable {
//                           ^^^^^^^^^ UndefinedClass: Class App\Countable does not exist
    public function isEmpty(): bool;
}
===expect===
