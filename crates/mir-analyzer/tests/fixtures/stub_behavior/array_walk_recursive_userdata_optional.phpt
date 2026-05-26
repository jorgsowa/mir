===description===
array_walk_recursive userdata parameter is optional
===file===
<?php
$items = ['a', ['b', 'c']];
// $userdata parameter has a default — calling with 2 args must not produce any error
array_walk_recursive($items, static function ($value, $key): void { /* … */ });
===expect===
