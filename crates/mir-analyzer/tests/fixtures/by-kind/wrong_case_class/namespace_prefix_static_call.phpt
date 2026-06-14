===description===
Wrong case in namespace prefix segment of a static call is reported.
===file===
<?php
namespace MyApp\Service;
class Greeter {
    public static function hello(): void {}
}

namespace Client;
\myapp\service\Greeter::hello();
===expect===
WrongCaseClass@8:1-8:23: Class name 'myapp\service\Greeter' has incorrect casing; use 'MyApp\Service\Greeter'
