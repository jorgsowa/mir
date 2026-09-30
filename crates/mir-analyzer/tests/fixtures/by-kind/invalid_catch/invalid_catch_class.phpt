===description===
Invalid catch class
===config===
suppress=UnusedVariable
===file===
<?php
class A {}
try {
    $worked = true;
}
catch (A $e) {}
//     ^ InvalidCatch: Caught type 'A' does not extend Throwable
===expect===
