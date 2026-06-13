===description===
MissingClosureReturnType does NOT fire when the closure has a @return docblock
immediately before the function keyword.
===file===
<?php
$a =
    /** @return string */
    function() {
        return "foo";
    };
===expect===
