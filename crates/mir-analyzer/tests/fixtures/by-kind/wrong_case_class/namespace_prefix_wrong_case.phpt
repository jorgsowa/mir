===description===
Wrong case in namespace prefix segment is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
$x = new \myapp\service\UserService();
===expect===
WrongCaseClass@6:10-6:36: Class name 'myapp\service\UserService' has incorrect casing; use 'MyApp\Service\UserService'
