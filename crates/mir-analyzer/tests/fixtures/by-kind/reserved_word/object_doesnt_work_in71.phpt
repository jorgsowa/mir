===description===
Object doesnt work in71
===ignore===
TODO
===file===
<?php
function foo(): object {
    return new stdClass();
}
===expect===
