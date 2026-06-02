===description===
Wrong return type in namespace1
===file===
<?php
namespace bar;

function fooFoo(): string {
    return 5;
}
===expect===
InvalidReturnType@5:5-5:14: Return type '5' is not compatible with declared 'string'
