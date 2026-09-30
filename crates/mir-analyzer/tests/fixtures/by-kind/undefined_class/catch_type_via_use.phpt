===description===
catch type via use
===config===
suppress=MissingThrowsDocblock,UnusedVariable,UnusedFunction
===file===
<?php
use Vendor\Missing\MyException;
function f(): void {
    try {
        throw new \Exception();
    } catch (MyException $e) {
//           ^^^^^^^^^^^ UndefinedClass: Class Vendor\Missing\MyException does not exist
    }
}
===expect===
