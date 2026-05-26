===description===
Catch does not return
===file===
<?php
function missing_return() : bool {
    try {
    } finally {
    }
}
===expect===
InvalidReturnType
===ignore===
TODO
