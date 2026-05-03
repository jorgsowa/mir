===description===
catch type via use
===file===
<?php
use Vendor\Missing\MyException;
function f(): void {
    try {
        throw new \Exception();
    } catch (MyException $e) {
    }
}
===expect===
UnusedVariable@1:0: Variable $e is never read
UndefinedClass@6:13: Class Vendor\Missing\MyException does not exist
===ignore===
TODO
