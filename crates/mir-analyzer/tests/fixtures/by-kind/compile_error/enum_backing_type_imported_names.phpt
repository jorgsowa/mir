===description===
A class named as an enum backing type is resolved through `use` imports, and `self` names the enum.
===config===
<mir>
  <issueHandlers>
    <UndefinedClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:single.php===
<?php
namespace App;

use Other\Bar;

enum Single: Bar {
//           ^^^ CompileError: Enum backing type must be int or string, Other\Bar given [invalid-enum-backing-type]
    case X = 1;
}

enum Itself: self {
//           ^^^^ CompileError: Enum backing type must be int or string, App\Itself given [invalid-enum-backing-type]
    case X = 1;
}
===file:union.php===
<?php
namespace App;

use Other\Bar;

enum Union: Bar|int {
//          ^^^^^^^ CompileError: Enum backing type must be int or string, Other\Bar|int given [invalid-enum-backing-type]
    case X = 1;
}
===file:nullable.php===
<?php
namespace App;

use Other\Bar;

enum Nullable: ?Bar {
//             ^^^^ CompileError: Enum backing type must be int or string, ?Other\Bar given [invalid-enum-backing-type]
    case X = 1;
}
