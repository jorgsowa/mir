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
InvalidCatch@6:8-6:9: Caught type 'A' does not extend Throwable
