===description===
a use-function import does not make an unrelated same-named type hint resolve against its target
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                          ^^^ UndefinedClass: Class App\foo does not exist
    {
//  ^ +1:5 InvalidReturnType: Return type 'void' is not compatible with declared 'App\foo'
    }
}
===expect===
