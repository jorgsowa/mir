===description===
cross file since 7 2 interface constant available
===config===
<mir>
  <phpVersion>7.2</phpVersion>
</mir>
===file:DateHelper.php===
<?php
function get_atom_format(): void {
    echo DateTimeInterface::ATOM;
}
===file:App.php===
<?php
get_atom_format();
