===description===
A class import must not change a function declaration's canonical FQN.
===config===
suppress=UnusedFunction
===file:Other.php===
<?php
namespace Other;

class helper {}

function helper(): int {
    return 1;
}
===file:Main.php===
<?php
namespace App;

use Other\helper;

function helper(): string {
    return 1;
}
===expect===
Main.php: InvalidReturnType@7:4-7:13: Return type '1' is not compatible with declared 'string'
