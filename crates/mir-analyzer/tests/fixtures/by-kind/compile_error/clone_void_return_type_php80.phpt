===description===
From PHP 8.0 `__clone()` may declare `void`.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
class A {
    function __clone(): void {}
}
