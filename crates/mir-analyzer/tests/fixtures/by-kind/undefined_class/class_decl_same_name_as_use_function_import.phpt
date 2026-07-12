===description===
class declaration is registered under its own namespace, not shadowed by a use-function import of the same short name
===config===
suppress=UnusedFunction,UnusedClass
===file:Helpers.php===
<?php
namespace App\Helpers;
function foo(): void {}
===file:Main.php===
<?php
namespace App;
use function App\Helpers\foo;
class foo {}
class Widget
{
    public function make(): foo
    {
        return new foo();
    }
}
===expect===
