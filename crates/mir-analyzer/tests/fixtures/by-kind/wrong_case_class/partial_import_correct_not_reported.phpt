===description===
Correct partial namespace import usage is not reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
use MyApp\Service;
$x = new Service\UserService();
===expect===
