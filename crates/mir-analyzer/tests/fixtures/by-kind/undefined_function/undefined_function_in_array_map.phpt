===description===
Undefined function in array map
===file===
<?php
array_map(
    "undefined_function",
//  ^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function undefined_function() is not defined
    [1, 2, 3]
);
