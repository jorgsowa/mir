===description===
builtinFunctioninvalidArgumentWithDeclareStrictTypes
===file===
<?php declare(strict_types=1);
                    $s = substr(5, 4);
===expect===
InvalidArgument@2:32: Argument $string of substr() expects 'string', got '5'
