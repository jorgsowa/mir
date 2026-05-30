===description===
Undefined function in array map
===file===
<?php
array_map(
    "undefined_function",
    [1, 2, 3]
);
===expect===
UndefinedFunction@3:5-3:25: Function undefined_function() is not defined
