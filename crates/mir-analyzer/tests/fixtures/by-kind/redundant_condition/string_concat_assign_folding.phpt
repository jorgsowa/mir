===description===
String concat-assign (.=) of literals folds to a literal result
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $s = "Hello";
    $s .= " World";
    /** @mir-check $s is "Hello World" */
    $_ = $s;

    $key = "user_";
    $key .= "name";
    /** @mir-check $key is "user_name" */
    $_ = $key;
}
===expect===
