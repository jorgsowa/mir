===description===
UndefinedDocblockClass does NOT fire when the referenced class exists.
===config===
suppress=UnusedParam
===file===
<?php
class MyExistingClass {}

/** @return MyExistingClass */
function getIt(): mixed {
    return new MyExistingClass();
}

/**
 * @param MyExistingClass $obj
 */
function takeIt($obj): void {}

===expect===
