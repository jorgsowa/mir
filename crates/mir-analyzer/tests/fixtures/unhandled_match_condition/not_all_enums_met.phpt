===description===
notAllEnumsMet
===file===
<?php
/**
 * @param "foo"|"bar" $foo
 */
function foo(string $foo): string {
    return match ($foo) {
        "foo" => "foo",
    };
}
===expect===
UnhandledMatchCondition
===ignore===
TODO
