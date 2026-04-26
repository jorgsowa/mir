===config===
php_version=7.1
===file:DateHelper.php===
<?php
function get_atom_format(): void {
    echo DateTimeInterface::ATOM;
}
===file:App.php===
<?php
get_atom_format();
===expect===
DateHelper.php: UndefinedConstant: Constant DateTimeInterface::ATOM is not defined
