===description===
A typed parameter made nullable only by its `null` default, which PHP 8.4 deprecates.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

interface Shape {}
interface Named {}

function plain(int $a = null): void {}
//             ^^^^^^^^^^^^^ ImplicitlyNullableParameter: App\plain(): Implicitly marking parameter $a as nullable is deprecated, the explicit nullable type must be used instead
function union(int|string $a = null): void {}
//             ^^^^^^^^^^^^^^^^^^^^ ImplicitlyNullableParameter: App\union(): Implicitly marking parameter $a as nullable is deprecated, the explicit nullable type must be used instead
function intersection(Shape&Named $a = null): void {}
//                    ^^^^^^^^^^^^^^^^^^^^^ ImplicitlyNullableParameter: App\intersection(): Implicitly marking parameter $a as nullable is deprecated, the explicit nullable type must be used instead
function explicit(?int $a = null, int|null $b = null, mixed $c = null): void {}
function typedDefault(int $a = 0): void {}

class Repository {
    public function find(string $id = NULL): void {}
//                       ^^^^^^^^^^^^^^^^^ ImplicitlyNullableParameter: App\Repository::find(): Implicitly marking parameter $id as nullable is deprecated, the explicit nullable type must be used instead
}
