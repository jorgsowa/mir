===description===
Array filter third arg will not be used when second null
===file===
<?php
array_filter( $arg, null, ARRAY_FILTER_USE_BOTH );
===expect===
InvalidArgument
===ignore===
TODO
