===description===
returnByReferenceNonVariableInClosure
===file===
<?php
function &(): int {
    return 45;
};

===expect===
NonVariableReferenceReturn
===ignore===
TODO
