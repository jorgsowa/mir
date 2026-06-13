===description===
MissingClosureReturnType does NOT fire when the closure has a return type annotation.
===file===
<?php
$a = function(): string {
    return "foo";
};
===expect===
