===description===
A class used only in a catch clause is pre-loaded via PSR-4 so Pass-2 does not emit
a false-positive UndefinedClass for the caught exception type.
===config===
suppress=UnusedVariable
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/DomainException.php===
<?php
namespace App;
class DomainException extends \Exception {}
===file:Service.php===
<?php
namespace App;
function run(): void {
    try {
        // work
    } catch (DomainException $e) {
        // handle
    }
}
===expect===
