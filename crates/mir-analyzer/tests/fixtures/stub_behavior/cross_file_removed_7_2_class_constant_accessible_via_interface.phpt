===description===
cross file removed 7 2 class constant accessible via interface
===config===
<mir>
  <phpVersion>7.2</phpVersion>
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
