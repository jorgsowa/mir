===description===
a use-function import does not make an unrelated same-named type hint resolve against its target
===config===
suppress=UnusedFunction
===file:Helpers.php===
<?php
namespace App\Helpers;
function foo(): void {}
===file:Main.php===
<?php
namespace App;
use function App\Helpers\foo;
class Widget {
    public function make(): foo
    {
    }
}
===expect===
Main.php: UndefinedClass@5:28-5:31: Class App\foo does not exist
Main.php: InvalidReturnType@6:4-7:5: Return type 'void' is not compatible with declared 'App\foo'
