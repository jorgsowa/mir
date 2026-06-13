===description===
Missing closure return type
===file===
<?php
$a = function() {
    return "foo";
};
===expect===
MissingClosureReturnType@2:6-4:7: Closure has no return type annotation
