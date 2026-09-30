===description===
InvalidNamedArguments fires once for each named argument passed to a @no-named-arguments function.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @no-named-arguments
 */
function create(string $name, int $count, bool $active): void {}

create(name: "test", count: 5, active: true);
//     ^^^^^^^^^^^^ InvalidNamedArguments: create() does not accept named arguments
//                   ^^^^^^^^ InvalidNamedArguments: create() does not accept named arguments
//                             ^^^^^^^^^^^^ InvalidNamedArguments: create() does not accept named arguments
===expect===
