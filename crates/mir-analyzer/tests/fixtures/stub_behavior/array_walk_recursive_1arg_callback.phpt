===description===
array_walk_recursive accepts a 1-arg callback (value only)
===file===
<?php
$items = [['a', 'b'], ['c']];
array_walk_recursive($items, static function ($value): void { echo $value; });
===expect===
