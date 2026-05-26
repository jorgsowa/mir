===description===
array_walk userdata parameter is optional
===file===
<?php
$items = ['a', 'b'];
// $userdata parameter has a default — calling with 2 args must not produce any error
array_walk($items, static function ($value, $key): void { /* … */ });
===expect===
