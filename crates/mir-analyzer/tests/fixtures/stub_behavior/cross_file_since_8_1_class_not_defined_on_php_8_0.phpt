===description===
cross file since 8 1 class not defined on php 8 0
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file:Async.php===
<?php
function make_fiber(callable $fn): void {
    new Fiber($fn);
//      ^^^^^ UndefinedClass: Class Fiber does not exist
}
===file:App.php===
<?php
make_fiber(function (): void {});
===expect===
