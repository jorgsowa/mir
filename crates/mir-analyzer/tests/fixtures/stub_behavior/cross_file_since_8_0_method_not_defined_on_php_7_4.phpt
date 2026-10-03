===description===
cross file since 8 0 method not defined on php 7 4
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file:DateHelper.php===
<?php
function from_interface(\DateTimeInterface $dt): void {
    DateTimeImmutable::createFromInterface($dt);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method DateTimeImmutable::createFromInterface() does not exist
}
===file:App.php===
<?php
from_interface(new DateTime());
===expect===
