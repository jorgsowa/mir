===description===
Array filter third arg will not be used when second null
===ignore===
TODO
===file===
<?php
array_filter( $arg, null, ARRAY_FILTER_USE_BOTH );
===expect===
UndefinedVariable@2:15-2:19: Variable $arg is not defined
