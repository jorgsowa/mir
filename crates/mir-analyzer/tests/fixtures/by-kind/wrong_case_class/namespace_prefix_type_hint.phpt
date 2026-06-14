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
===expect===
WrongCaseClass@6:17-6:43: Class name 'myapp\service\UserService' has incorrect casing; use 'MyApp\Service\UserService'
