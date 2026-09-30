===description===
Array filter third arg invalid behaves like0
===config===
suppress=MixedArgument
===file===
<?php
array_filter( $arg, "strlen", 3 );
//            ^^^^ UndefinedVariable: Variable $arg is not defined
===expect===
