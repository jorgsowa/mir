===description===
A composer autoload root and an include seed together: the seed's include closure is analyzed alongside
===config===
<mir>
  <projectFiles>
    <file name="legacy/a.module"/>
  </projectFiles>
  <fileExtensions>
    <extension name="php"/>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Main.php===
<?php
namespace App;
class Main { public function run(): int { return 1; } }
===file:legacy/a.module===
<?php
require __DIR__ . '/b.inc';
function a_hook(): int { return b(); }
===file:legacy/b.inc===
<?php
function b(): int { return 1; }
