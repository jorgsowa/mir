===description===
cross file since 7 2 interface constant not before
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
DateHelper.php: UndefinedConstant@3:9: Constant DateTimeInterface::ATOM is not defined
===ignore===
TODO
