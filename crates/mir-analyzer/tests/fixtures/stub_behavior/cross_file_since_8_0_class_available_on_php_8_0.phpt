===description===
cross file since 8 0 class available on php 8 0
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file:Cache.php===
<?php
function make_weak_cache(): void {
    new WeakMap();
}
===file:App.php===
<?php
make_weak_cache();
===expect===
