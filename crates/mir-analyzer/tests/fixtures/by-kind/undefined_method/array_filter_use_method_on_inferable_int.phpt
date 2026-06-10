===description===
Array filter use method on inferable int
===file===
<?php
$a = array_filter([1, 2, 3, 4], function ($i) { return $i->foo(); });
===expect===
MixedMethodCall@2:56-2:65: Method foo() called on mixed type
