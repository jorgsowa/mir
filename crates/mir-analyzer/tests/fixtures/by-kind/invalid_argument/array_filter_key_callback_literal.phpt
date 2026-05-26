===description===
Array filter key callback literal
===file===
<?php
array_filter(["a" => 5, "b" => 12, "c" => null], "abs", ARRAY_FILTER_USE_KEY);
===expect===
InvalidArgument
===ignore===
TODO
