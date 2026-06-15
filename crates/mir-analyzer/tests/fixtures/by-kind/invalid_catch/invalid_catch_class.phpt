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
===expect===
InvalidCatch@6:7-6:8: Caught type 'A' does not extend Throwable
