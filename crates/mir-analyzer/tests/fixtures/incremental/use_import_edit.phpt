===description===
Retargeting a `use` alias re-infers the return type of a method that
resolves it, even though the method body is unchanged.
===file:Lib.php===
<?php
namespace Lib;
class One { public function make(): int { return 1; } }
class Two { public function make(): string { return ''; } }
===file:F.php===
<?php
namespace App;
use Lib\One as Dep;
class F {
    public function a() { return new Dep(); }
    public function b() { return (new Dep())->make(); }
}
===file:C.php===
<?php
namespace App;
function c(): int { return (new F())->b(); }
===edit:F.php===
<?php
namespace App;
use Lib\Two as Dep;
class F {
    public function a() { return new Dep(); }
    public function b() { return (new Dep())->make(); }
}
===expect===
C.php: InvalidReturnType@3:20-3:42: Return type 'string' is not compatible with declared 'int'
