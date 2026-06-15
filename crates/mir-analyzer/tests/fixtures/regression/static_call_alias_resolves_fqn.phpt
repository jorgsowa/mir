===description===
Alias::method() where Alias is a use-imported class resolves both the class
FQN (ClassReference on the Alias token) and the method (StaticCall with the
correct declaring class). Return type and no UndefinedMethod confirm resolution.
===config===
suppress=UnusedFunction,UnusedParam,UnusedVariable
===file:Str.php===
<?php
namespace Illuminate\Support;
class Str {
    public static function camel(string $value): string { return $value; }
}
===file:Gate.php===
<?php
use Illuminate\Support\Str;
function test(string $ability): string {
    $result = Str::camel($ability);
    /** @mir-check $result is string */
    return $result;
}
===expect===
