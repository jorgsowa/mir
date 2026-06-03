===description===
cross file removed 7 2 class constant accessible via interface
===config===
php_version=7.2
===file:DateHelper.php===
<?php
function get_atom_format(): void {
    echo DateTime::ATOM;
}
===file:App.php===
<?php
get_atom_format();
===expect===
