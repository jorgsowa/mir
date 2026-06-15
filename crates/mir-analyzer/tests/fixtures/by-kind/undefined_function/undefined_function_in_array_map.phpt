===description===
Undefined function in array map
===file===
<?php
array_map(
    "undefined_function",
    [1, 2, 3]
);
===expect===
UndefinedFunction@3:4-3:24: Function undefined_function() is not defined
