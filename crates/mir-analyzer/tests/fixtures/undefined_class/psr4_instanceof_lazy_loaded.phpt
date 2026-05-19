===description===
A class used only in an instanceof expression is pre-loaded via PSR-4 so Pass-2 does
not emit a false-positive UndefinedClass.
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Response.php===
<?php
namespace App;
class Response {}
===file:Checker.php===
<?php
namespace App;
function check(mixed $val): bool {
    return $val instanceof Response;
}
===expect===
