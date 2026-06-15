===description===
Array filter third arg invalid behaves like0
===config===
suppress=MixedArgument
===file===
<?php
array_filter( $arg, "strlen", 3 );
===expect===
UndefinedVariable@2:14-2:18: Variable $arg is not defined
