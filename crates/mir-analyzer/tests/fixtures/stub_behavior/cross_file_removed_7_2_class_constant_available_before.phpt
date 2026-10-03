===description===
cross file removed 7 2 class constant available before
===config===
<mir>
  <phpVersion>7.1</phpVersion>
</mir>
===file:DateHelper.php===
<?php
function get_atom_format(): void {
    echo DateTime::ATOM;
}
===file:App.php===
<?php
get_atom_format();
===expect===
