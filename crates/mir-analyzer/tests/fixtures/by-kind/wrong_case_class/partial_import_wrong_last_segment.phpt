===description===
Partial namespace import followed by wrong-case last segment is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
use MyApp\Service;
$x = new Service\userservice();
===expect===
WrongCaseClass@7:9-7:28: Class name 'userservice' has incorrect casing; use 'UserService'
