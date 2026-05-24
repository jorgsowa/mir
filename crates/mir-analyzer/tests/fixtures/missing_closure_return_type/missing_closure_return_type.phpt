===description===
missingClosureReturnType
===file===
<?php
$a = function() {
    return "foo";
};
===expect===
MissingClosureReturnType
===ignore===
TODO
