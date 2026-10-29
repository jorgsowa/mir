===description===
From PHP 8.0 an anonymous class is named after its parent or first interface in compile errors.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:abstract.php===
<?php
namespace App;

class P {}

$a = new class extends P {
    abstract function a();
//  ^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class App\P@anonymous contains 2 abstract methods and must therefore be declared abstract or implement the remaining methods (App\P@anonymous::a, App\P@anonymous::b) [abstract-anonymous-class-method]
    abstract function b();
};
===file:readonly.php===
<?php
namespace App;

interface I {}

$b = new class implements I {
    public readonly $q;
//  ^^^^^^^^^^^^^^^^^^ CompileError: Readonly property App\I@anonymous::$q must have type [untyped-readonly-property]
};
