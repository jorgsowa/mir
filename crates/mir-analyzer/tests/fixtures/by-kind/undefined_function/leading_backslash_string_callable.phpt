===description===
A string callable may start with a backslash (`'\intval'`, `'\App\mine'`): it names
the same function without the backslash, and a name that exists nowhere is still
reported without it.
===file===
<?php
namespace App;

function mine(int $x): int {
    return $x;
}

function test(): void {
    array_map('\intval', ['1']);
    array_map('\App\mine', [1]);
    array_map('App\mine', [1]);
    array_map('\nope', [1]);
//            ^^^^^^^ UndefinedFunction: Function nope() is not defined
    array_map('\App\nope', [1]);
//            ^^^^^^^^^^^ UndefinedFunction: Function App\nope() is not defined
    usort($a, '\strcmp');
    call_user_func('\strlen', 'a');
    array_map('mine', [1]);
//            ^^^^^^ UndefinedFunction: Function mine() is not defined
}
