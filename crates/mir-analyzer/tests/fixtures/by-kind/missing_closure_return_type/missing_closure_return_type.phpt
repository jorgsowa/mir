===description===
Missing closure return type
===config===
suppress=UnusedVariable
===file===
<?php
$a = function() {
    return "foo";
};
===expect===
MissingClosureReturnType@2:5-4:1: Closure has no return type annotation
