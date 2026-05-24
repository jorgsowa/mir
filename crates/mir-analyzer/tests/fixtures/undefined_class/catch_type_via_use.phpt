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
MissingThrowsDocblock@5:9: Exception Exception is thrown but not declared in @throws
UnusedVariable@6:13: Variable $e is never read
UndefinedClass@6:14: Class Vendor\Missing\MyException does not exist
