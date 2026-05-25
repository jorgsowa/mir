===description===
Array filter callback validation third arg0
===file===
<?php
/**
 * @var array<int, string|int|float> $arg
 */
array_filter($arg, "abs", 0);
===expect===
InvalidArgument
===ignore===
TODO
