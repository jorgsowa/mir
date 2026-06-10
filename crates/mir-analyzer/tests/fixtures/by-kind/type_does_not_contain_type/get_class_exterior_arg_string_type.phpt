===description===
Get class exterior arg string type
===ignore===
TODO
===file===
<?php
/** @return void */
function foo(Exception $e) {
    switch (get_class($e)) {
        case "InvalidArgumentException":
            break;
    }
}
===expect===
