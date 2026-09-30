===description===
a function declared inside a closure body is local, not a file-scope declaration
===config===
suppress=MissingClosureReturnType,UnusedVariable
===file===
<?php
$fn = function () {
    function leaked_from_closure() {}
};

leaked_from_closure();
//<^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function leaked_from_closure() is not defined
===expect===
