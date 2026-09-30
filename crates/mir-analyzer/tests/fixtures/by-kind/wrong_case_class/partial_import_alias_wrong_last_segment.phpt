===description===
Partial namespace import with alias and wrong-case last segment is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
use MyApp\Service as Svc;
$x = new Svc\userservice();
//       ^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'userservice' has incorrect casing; use 'UserService'
===expect===
