===description===
UndefinedTrait names a namespaced trait by its fully qualified name
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

class Named {
    use Missing;
//      ^^^^^^^ UndefinedTrait: Trait App\Missing does not exist
}

$x = new class {
    use Missing;
//      ^^^^^^^ UndefinedTrait: Trait App\Missing does not exist
};
