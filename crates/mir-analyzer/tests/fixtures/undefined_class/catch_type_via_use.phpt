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
UndefinedClass: Class Vendor\Missing\MyException does not exist
UnusedVariable: Variable $e is never read
