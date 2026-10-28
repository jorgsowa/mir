===description===
Before PHP 8.2 creating a dynamic property is silent.
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
class Plain {}

function write(Plain $a): void {
    $a->x = 1;
}
