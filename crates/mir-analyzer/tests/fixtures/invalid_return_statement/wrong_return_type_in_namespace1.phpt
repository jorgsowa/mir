===description===
wrongReturnTypeInNamespace1
===file===
<?php
namespace bar;

function fooFoo(): string {
    return 5;
}
===expect===
InvalidReturnStatement
===ignore===
TODO
