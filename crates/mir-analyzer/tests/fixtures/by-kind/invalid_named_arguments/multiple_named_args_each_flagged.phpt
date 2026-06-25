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
===expect===
InvalidNamedArguments@7:7-7:19: create() does not accept named arguments
InvalidNamedArguments@7:21-7:29: create() does not accept named arguments
InvalidNamedArguments@7:31-7:43: create() does not accept named arguments
