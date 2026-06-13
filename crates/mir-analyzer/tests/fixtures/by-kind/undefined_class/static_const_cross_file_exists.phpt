===description===
Foo::CONST cross file exists — no UndefinedClass when class is defined in another file
===config===
suppress=MixedReturnStatement
===file:Config.php===
<?php
namespace App;
class Config {
    const VERSION = '1.0';
}
===file:Reader.php===
<?php
use App\Config;
function getVersion(): string {
    return Config::VERSION;
}
===expect===
