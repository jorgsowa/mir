===description===
array_walk accepts a 1-arg callback (value only)
===file===
<?php
$items = ['a', 'b'];
array_walk($items, static function ($value): void { echo $value; });
===expect===
