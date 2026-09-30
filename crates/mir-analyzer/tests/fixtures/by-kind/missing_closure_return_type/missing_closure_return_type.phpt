===description===
Missing closure return type
===config===
suppress=UnusedVariable
===file===
<?php
$a = function() {
//   ^ +2:1 MissingClosureReturnType: Closure has no return type annotation
    return "foo";
};
===expect===
