===description===
Passing an undefined variable to a by-reference param via a named
argument (reordered relative to its declared position) still defines it
— premark_byref_arg_vars used to assume args[i] always feeds params[i],
missing the rename/reorder that named arguments introduce.
===config===
suppress=MissingReturnType,MissingParamType,UnusedParam
===file===
<?php
function f($a, &$b) {
    $b = 5;
}

f(b: $result, a: 1);
echo $result;
===expect===
