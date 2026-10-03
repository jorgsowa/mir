===description===
A class import must not change a function declaration's canonical FQN.
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Other.php===
<?php
namespace Other;

class helper {}

function helper(): int {
    return 1;
}
===file:Main.php===
<?php
namespace App;

use Other\helper;

function helper(): string {
    return 1;
//  ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'string'
}
===expect===
