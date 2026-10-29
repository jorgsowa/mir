===description===
Before PHP 8.0 a `mixed` hint names a class; each hint is one `UndefinedClass` and the value is typed `mixed`, so arguments, returns and assignments are not checked against the missing class.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
namespace App;

final class Box {
    /** @var mixed */
    private $v;

    public function set(mixed $v): void { $this->v = $v; }
//                      ^^^^^ UndefinedClass: Class App\mixed does not exist

    public function get(): mixed { return $this->v; }
//                         ^^^^^ UndefinedClass: Class App\mixed does not exist
}

function first(mixed $x): string { return is_string($x) ? $x : ''; }
//             ^^^^^ UndefinedClass: Class App\mixed does not exist

first('a');
(new Box())->set(1);
