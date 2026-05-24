===description===
objectDoesntWorkIn71
===file===
<?php
function foo(): object {
    return new stdClass();
}
===expect===
ReservedWord
===ignore===
TODO
