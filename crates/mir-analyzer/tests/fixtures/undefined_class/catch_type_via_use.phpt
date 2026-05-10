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
MissingThrowsDocblock@5:8: Exception Exception is thrown but not declared in @throws
UnusedVariable@6:12: Variable $e is never read
UndefinedClass@6:13: Class Vendor\Missing\MyException does not exist
