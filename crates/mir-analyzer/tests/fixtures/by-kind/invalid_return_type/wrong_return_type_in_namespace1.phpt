===description===
Wrong return type in namespace1
===file===
<?php
namespace bar;

function fooFoo(): string {
    return 5;
//  ^^^^^^^^^ InvalidReturnType: Return type '5' is not compatible with declared 'string'
}
===expect===
