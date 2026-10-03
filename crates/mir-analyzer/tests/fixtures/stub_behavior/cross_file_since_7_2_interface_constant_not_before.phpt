===description===
cross file since 7 2 interface constant not before
===config===
<mir>
  <phpVersion>7.1</phpVersion>
</mir>
===file:DateHelper.php===
<?php
function get_atom_format(): void {
    echo DateTimeInterface::ATOM;
//       ^^^^^^^^^^^^^^^^^^^^^^^ UndefinedConstant: Constant DateTimeInterface::ATOM is not defined
}
===file:App.php===
<?php
get_atom_format();
===expect===
