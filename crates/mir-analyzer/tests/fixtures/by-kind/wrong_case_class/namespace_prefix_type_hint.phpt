===description===
Wrong case in namespace prefix segment of a type hint is reported.
===config===
suppress=UnusedParam
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
function handle(\myapp\service\UserService $s): void {}
//              ^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'myapp\service\UserService' has incorrect casing; use 'MyApp\Service\UserService'
===expect===
