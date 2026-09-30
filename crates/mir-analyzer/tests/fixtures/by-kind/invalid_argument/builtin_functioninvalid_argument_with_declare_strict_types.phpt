===description===
Builtin functioninvalid argument with declare strict types
===config===
suppress=UnusedVariable
===file===
<?php declare(strict_types=1);
                    $s = substr(5, 4);
//                              ^ InvalidArgument: Argument $string of substr() expects 'string', got '5'
===expect===
