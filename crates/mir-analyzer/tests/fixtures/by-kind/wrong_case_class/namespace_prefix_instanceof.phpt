===description===
Wrong case in namespace prefix segment of instanceof is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
$obj = new \MyApp\Service\UserService();
$x = $obj instanceof \myapp\service\UserService;
//                   ^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'myapp\service\UserService' has incorrect casing; use 'MyApp\Service\UserService'
===expect===
