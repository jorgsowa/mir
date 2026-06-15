===description===
cross file since 8 0 method not defined on php 7 4
===config===
php_version=7.4
===file:DateHelper.php===
<?php
function from_interface(\DateTimeInterface $dt): void {
    DateTimeImmutable::createFromInterface($dt);
}
===file:App.php===
<?php
from_interface(new DateTime());
===expect===
DateHelper.php: UndefinedMethod@3:4-3:47: Method DateTimeImmutable::createFromInterface() does not exist
