===description===
A typed parameter made nullable only by its `null` default, which PHP 8.3 accepts.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
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
function union(int|string $a = null): void {}
function intersection(Shape&Named $a = null): void {}
function explicit(?int $a = null, int|null $b = null, mixed $c = null): void {}
function typedDefault(int $a = 0): void {}

class Repository {
    public function find(string $id = NULL): void {}
}

$closure = fn(int $a = null): int => 1;
