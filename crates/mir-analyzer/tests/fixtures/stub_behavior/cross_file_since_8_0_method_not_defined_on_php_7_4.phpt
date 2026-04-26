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
DateHelper.php: UndefinedMethod: Method DateTimeImmutable::createFromInterface() does not exist
