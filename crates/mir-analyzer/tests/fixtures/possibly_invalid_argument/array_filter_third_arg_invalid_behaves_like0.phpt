===description===
Array filter third arg invalid behaves like0
===file===
<?php
array_filter( $arg, "strlen", 3 );
===expect===
PossiblyInvalidArgument
===ignore===
TODO
