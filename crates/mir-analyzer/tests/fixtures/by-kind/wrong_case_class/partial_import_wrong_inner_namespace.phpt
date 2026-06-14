===description===
Partial namespace import with wrong inner namespace segment reports the fully resolved path.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Deep\Service;
class UserService {}

namespace Client;
use MyApp\Deep;
$x = new Deep\service\UserService();
===expect===
WrongCaseClass@7:10-7:34: Class name 'MyApp\Deep\service\UserService' has incorrect casing; use 'MyApp\Deep\Service\UserService'
